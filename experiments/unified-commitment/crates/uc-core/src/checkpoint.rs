use crate::{
    Binding, CheckpointRef, Decode, History, Log, LogError, Outcome, Point, Position, crc32c,
    envelope::{Cursor, blob},
    sha256, sync_directory,
};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

fn cache(bindings: &BTreeMap<[u8; 16], Binding>, offset: u64) -> Vec<u8> {
    let mut out = Vec::new();
    let entries: Vec<_> = bindings.iter().filter(|(_, b)| b.offset < offset).collect();
    out.extend((entries.len() as u64).to_le_bytes());
    for (id, b) in entries {
        out.extend(id);
        out.extend(b.event.0);
        out.extend(b.hash);
        blob(&mut out, &b.normalized);
        match b.outcome {
            Outcome::Committed {
                sequence,
                physical_ordinal,
                durability_time,
                achieved,
            } => {
                out.push(1);
                out.extend(sequence.to_le_bytes());
                out.extend(physical_ordinal.to_le_bytes());
                out.extend(durability_time.to_le_bytes());
                out.push(achieved as u8);
            }
            _ => out.push(0),
        }
    }
    out
}
pub(crate) fn write<S>(
    log: &Log<S>,
    encoder: impl FnOnce(&S) -> Result<Vec<u8>, String>,
) -> Result<CheckpointRef, LogError> {
    let position = log
        .position
        .ok_or_else(|| LogError::Invalid("checkpoint requires a commit".into()))?;
    let state = encoder(&log.state).map_err(LogError::Invalid)?;
    let mut bytes = b"UC1\0".to_vec();
    bytes.extend(position.sequence.to_le_bytes());
    bytes.extend(position.physical_ordinal.to_le_bytes());
    bytes.extend(position.offset.to_le_bytes());
    bytes.extend(sha256(&log.prefix[..position.offset as usize]));
    blob(&mut bytes, &state);
    blob(&mut bytes, &cache(&log.bindings, position.offset));
    bytes.extend(crc32c(&bytes).to_le_bytes());
    let path = log.directory.path.join(format!(
        "checkpoint-{}-{}.uc1",
        position.sequence, position.physical_ordinal
    ));
    log.writer.synchronize()?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    (log.hook)(Point::Checkpoint);
    sync_directory(&log.directory.path)?;
    Ok(CheckpointRef { path, position })
}
struct Parsed<'a> {
    position: Position,
    hash: &'a [u8],
    state: &'a [u8],
    cache: &'a [u8],
}
fn parse(bytes: &[u8]) -> Result<Parsed<'_>, String> {
    let n = bytes.len().checked_sub(4).ok_or("short checkpoint")?;
    if crc32c(&bytes[..n]) != u32::from_le_bytes(bytes[n..].try_into().unwrap()) {
        return Err("checkpoint CRC".into());
    }
    let mut c = Cursor(&bytes[..n]);
    if c.take(4)? != b"UC1\0" {
        return Err("checkpoint magic".into());
    }
    let position = Position {
        sequence: c.u64()?,
        physical_ordinal: c.u64()?,
        offset: c.u64()?,
    };
    let parsed = Parsed {
        position,
        hash: c.take(32)?,
        state: c.blob()?,
        cache: c.blob()?,
    };
    c.end()?;
    Ok(parsed)
}
pub(crate) fn restore<S: Default>(
    path: &Path,
    history: &mut History,
    decode: Decode<S>,
) -> Result<(S, Option<Position>), LogError> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(path)? {
        let p = entry?.path();
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !name.starts_with("checkpoint-") || !name.ends_with(".uc1") {
            continue;
        }
        if fs::metadata(&p)?.len() > crate::MAX_HISTORY {
            history
                .report
                .rejected_checkpoints
                .push((p, "checkpoint limit".into()));
            continue;
        }
        let bytes = fs::read(&p)?;
        match parse(&bytes) {
            Ok(parsed) => {
                if name
                    != format!(
                        "checkpoint-{}-{}.uc1",
                        parsed.position.sequence, parsed.position.physical_ordinal
                    )
                {
                    history
                        .report
                        .rejected_checkpoints
                        .push((p, "checkpoint filename/position mismatch".into()));
                    continue;
                }
                candidates.push((
                    parsed.position.sequence,
                    parsed.position.physical_ordinal,
                    p,
                    bytes,
                ));
            }
            Err(e) => history.report.rejected_checkpoints.push((p, e)),
        }
    }
    candidates.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
    for (_, _, path, bytes) in candidates {
        let cp = parse(&bytes).map_err(LogError::Invalid)?;
        if cp.position.offset > history.prefix.len() as u64 {
            return Err(LogError::HistoryShorterThanCheckpoint {
                offset: cp.position.offset,
            });
        }
        if !history.events.iter().any(|e| e.position == cp.position)
            || sha256(&history.prefix[..cp.position.offset as usize]) != cp.hash
        {
            history
                .report
                .rejected_checkpoints
                .push((path, "history position/prefix hash mismatch".into()));
            continue;
        }
        // Full scan remains authoritative. Compare only the cache's exact history prefix.
        if cache(&history.bindings, cp.position.offset) != cp.cache {
            return Err(LogError::CheckpointCacheMismatch);
        }
        match decode(cp.state) {
            Ok(state) => {
                history.report.checkpoint = Some(cp.position);
                return Ok((state, Some(cp.position)));
            }
            Err(e) => history
                .report
                .rejected_checkpoints
                .push((path, format!("state decode: {e}"))),
        }
    }
    Ok((S::default(), None))
}
