//! Bounded EXP-0003 single-writer RF1 history. No production compatibility promise.
mod checkpoint;
mod envelope;
mod recovery;
mod sha;
use exp1_record_format::{Body, IntegrityProfile, Record, encode};
pub use exp1_record_format::{Uuid, crc32c};
pub use recovery::{CommittedEvent, History, OpenReport, read_history};
pub use sha::sha256;
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const MAX_PAYLOAD: usize = 4 * 1024 * 1024;
pub const MAX_HISTORY: u64 = 1_073_741_824;
pub const MAX_RECORDS: u64 = 1_000_000;
pub const HISTORY_FILE: &str = "history.rf1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Durability {
    D1 = 1,
    D2 = 2,
}
impl Durability {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "d1" => Ok(Self::D1),
            "d2" => Ok(Self::D2),
            _ => Err("expected d1 or d2".into()),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::D1 => "unified D1 ordinary writes, no sync, no crash-survival claim",
            Self::D2 => {
                "unified D2 per-transaction sync (R5 placements); tested failure model: injected process termination and torn tails; platform durability contract evidence-pending; no OS-crash or power-loss claim"
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct Transaction {
    pub request_id: Uuid,
    pub payload: Vec<u8>,
    pub durability: Durability,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    RequestIdReuse,
    Uncommitted,
    Validation(String),
    Io(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Committed {
        sequence: u64,
        physical_ordinal: u64,
        durability_time: i64,
        achieved: Durability,
    },
    Rejected {
        reason: Rejection,
    },
    Indeterminate {
        request_id: Uuid,
    },
}
#[derive(Debug)]
pub enum LogError {
    Io { offset: u64, kind: io::ErrorKind },
    NotFound,
    Owned,
    Poisoned,
    Limit,
    Invalid(String),
    Corrupt { offset: u64, error: String },
    EnvelopeMismatch { physical_ordinal: u64 },
    HistoryShorterThanCheckpoint { offset: u64 },
    CheckpointCacheMismatch,
}
impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::HistoryShorterThanCheckpoint { offset } = self {
            write!(f, "history shorter than checkpoint at {offset}")
        } else {
            write!(f, "{self:?}")
        }
    }
}
impl std::error::Error for LogError {}
impl From<io::Error> for LogError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::NotFound {
            Self::NotFound
        } else {
            Self::Io {
                offset: 0,
                kind: e.kind(),
            }
        }
    }
}

/// The file's existence is irrelevant. Drop/OS process exit releases ownership.
pub struct Directory {
    path: PathBuf,
    _owner: File,
}
impl Directory {
    pub fn acquire(path: &Path) -> Result<Self, LogError> {
        if !fs::metadata(path)?.is_dir() {
            return Err(LogError::Invalid("storage path is not a directory".into()));
        }
        let owner = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.join("owner.lock"))?;
        match owner.try_lock() {
            Ok(()) => Ok(Self {
                path: path.to_owned(),
                _owner: owner,
            }),
            Err(std::fs::TryLockError::WouldBlock) => Err(LogError::Owned),
            Err(std::fs::TryLockError::Error(e)) => Err(e.into()),
        }
    }
}
/// Platform call boundary, not a claim about device or namespace survival.
pub fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // GENERIC_WRITE is required by FlushFileBuffers; BACKUP_SEMANTICS opens directories.
        OpenOptions::new()
            .access_mode(0x4000_0000)
            .custom_flags(0x0200_0000)
            .open(path)?
            .sync_all()
    }
    #[cfg(not(windows))]
    {
        File::open(path)?.sync_all()
    }
}

/// Injectable only at the real writer boundary. A short write is fail-stop, not retried.
pub trait Writer: Write {
    fn synchronize(&self) -> io::Result<()>;
}
impl Writer for File {
    fn synchronize(&self) -> io::Result<()> {
        self.sync_all()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Point {
    Binding,
    Reservation,
    Provisional,
    Final,
    Commit,
    Synced,
    Checkpoint,
}
impl Point {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "binding" => Ok(Self::Binding),
            "reservation" => Ok(Self::Reservation),
            "provisional" => Ok(Self::Provisional),
            "final" => Ok(Self::Final),
            "commit" => Ok(Self::Commit),
            "synced" => Ok(Self::Synced),
            "checkpoint" => Ok(Self::Checkpoint),
            _ => Err("unknown injection point".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Binding {
    event: Uuid,
    normalized: Vec<u8>,
    hash: [u8; 32],
    outcome: Outcome,
    offset: u64,
}
pub type Apply<S> = fn(&mut S, &[u8]) -> Result<(), String>;
pub type Decode<S> = fn(&[u8]) -> Result<S, String>;
pub struct Log<S> {
    directory: Directory,
    writer: Box<dyn Writer>,
    state: Arc<S>,
    apply: Apply<S>,
    bindings: BTreeMap<[u8; 16], Binding>,
    prefix: Vec<u8>,
    next_sequence: u64,
    ordinal: u64,
    position: Option<Position>,
    poisoned: bool,
    hook: Box<dyn Fn(Point)>,
    pub append_time: Duration,
    pub payload_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub sequence: u64,
    pub physical_ordinal: u64,
    pub offset: u64,
}
#[derive(Clone, Debug)]
pub struct CheckpointRef {
    pub path: PathBuf,
    pub position: Position,
}

impl<S: Clone + Default> Log<S> {
    /// Exclusively create a storage directory and history; never adopt an existing directory.
    pub fn create(
        path: &Path,
        apply: Apply<S>,
        decode: Decode<S>,
    ) -> Result<(Self, OpenReport), LogError> {
        fs::create_dir(path)?;
        let directory = Directory::acquire(path)?;
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(path.join(HISTORY_FILE))?;
        Self::recover(directory, file, apply, decode)
    }
    pub fn open(
        path: &Path,
        apply: Apply<S>,
        decode: Decode<S>,
    ) -> Result<(Self, OpenReport), LogError> {
        let history_path = path.join(HISTORY_FILE);
        // Check before acquiring ownership so a missing history creates no owner file.
        fs::metadata(&history_path)?;
        let directory = Directory::acquire(path)?;
        // Synchronize namespace on every open, including a file left by an interrupted creator.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&history_path)?;
        Self::recover(directory, file, apply, decode)
    }
    fn recover(
        directory: Directory,
        file: File,
        apply: Apply<S>,
        decode: Decode<S>,
    ) -> Result<(Self, OpenReport), LogError> {
        let path = &directory.path;
        let history_path = path.join(HISTORY_FILE);
        file.sync_all()?;
        sync_directory(path)?;
        let mut history = read_history(&history_path)?;
        let (mut state, checkpoint_position) = checkpoint::restore(path, &mut history, decode)?;
        for event in &history.events {
            if checkpoint_position.is_none_or(|p| event.position.offset > p.offset) {
                apply(&mut state, &event.payload).map_err(LogError::Invalid)?;
            }
        }
        if let Some(offset) = history.report.dropped_tail {
            file.set_len(offset)?;
            file.sync_all()?;
            history.report.tail_truncated_and_synced = true;
        }
        drop(file);
        let file = OpenOptions::new().append(true).open(&history_path)?;
        let position = history.events.last().map(|e| e.position);
        let next_sequence = history.high_water.checked_add(1).ok_or(LogError::Limit)?;
        let log = Self {
            directory,
            writer: Box::new(file),
            state: Arc::new(state),
            apply,
            bindings: history.bindings,
            prefix: history.prefix,
            next_sequence,
            ordinal: history.ordinal,
            position,
            poisoned: false,
            hook: Box::new(|_| {}),
            append_time: Duration::ZERO,
            payload_bytes: 0,
        };
        Ok((log, history.report))
    }
    pub fn snapshot(&self) -> Arc<S> {
        Arc::clone(&self.state)
    }
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }
    pub fn position(&self) -> Option<Position> {
        self.position
    }
    pub fn history_bytes(&self) -> u64 {
        self.prefix.len() as u64
    }
    pub fn next_request_id(&self) -> Uuid {
        identity(self.ordinal + 1, 0x41)
    }
    /// Test instrumentation observes exact call placements; never adds record metadata.
    pub fn set_hook(&mut self, hook: impl Fn(Point) + 'static) {
        self.hook = Box::new(hook);
    }
    /// Wrap the actual writer for deterministic short-write/error/sync injection.
    pub fn wrap_writer(&mut self, wrap: impl FnOnce(Box<dyn Writer>) -> Box<dyn Writer>) {
        struct Unavailable;
        impl Write for Unavailable {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("unavailable"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("unavailable"))
            }
        }
        impl Writer for Unavailable {
            fn synchronize(&self) -> io::Result<()> {
                Err(io::Error::other("unavailable"))
            }
        }
        self.writer = wrap(std::mem::replace(&mut self.writer, Box::new(Unavailable)));
    }
    pub fn commit(
        &mut self,
        txn: Transaction,
        validate: impl FnOnce(&S) -> Result<(), Rejection>,
    ) -> Result<Outcome, LogError> {
        if self.poisoned {
            return Err(LogError::Poisoned);
        }
        let normalized = envelope::normalize(&txn);
        if let Some(binding) = self.bindings.get(&txn.request_id.0) {
            // A previously bound request has precedence even when the changed request
            // exceeds the new-transaction limit. The retained bytes are necessarily bounded.
            return Ok(
                if normalized.as_ref().is_ok_and(|bytes| {
                    binding.hash == sha256(bytes) && binding.normalized == *bytes
                }) {
                    binding.outcome.clone()
                } else {
                    Outcome::Rejected {
                        reason: Rejection::RequestIdReuse,
                    }
                },
            );
        }
        let normalized = normalized?;
        txn.request_id
            .validate_v4()
            .map_err(|e| LogError::Invalid(format!("request: {e:?}")))?;
        if let Err(reason) = validate(&self.state) {
            return Ok(Outcome::Rejected { reason });
        }
        // Stage all payload effects before touching the history. Readers hold immutable Arcs.
        let mut next = self.state.as_ref().clone();
        if let Err(e) = (self.apply)(&mut next, &txn.payload) {
            return Ok(Outcome::Rejected {
                reason: Rejection::Validation(e),
            });
        }
        let sequence = self.next_sequence;
        let next_sequence = sequence.checked_add(1).ok_or(LogError::Limit)?;
        let last_ordinal = self.ordinal.checked_add(5).ok_or(LogError::Limit)?;
        if last_ordinal > MAX_RECORDS {
            return Err(LogError::Limit);
        }
        let event_id = identity(self.ordinal + 1, 0x45);
        let record = |i, body| Record {
            physical_ordinal: self.ordinal + i,
            integrity: IntegrityProfile::Crc32c,
            body,
        };
        let bodies = [
            Body::Binding {
                request_id: txn.request_id,
                event_id,
                normalized_request: normalized.clone(),
            },
            Body::Reservation {
                request_id: txn.request_id,
                event_id,
                sequence,
                high_water: sequence,
            },
            Body::Provisional {
                event_id,
                sequence,
                group_id: 0,
                member_index: 0,
                member_count: 1,
                stable_core: txn.payload.clone(),
            },
        ];
        let mut frames = Vec::new();
        for (i, body) in bodies.into_iter().enumerate() {
            frames.push(
                encode(&record(i as u64 + 1, body))
                    .map_err(|e| LogError::Invalid(format!("encode {e:?}")))?,
            );
        }
        // Exact final/commit sizes are independent of the later clock sample.
        let required = frames.iter().map(Vec::len).sum::<usize>() + 84 + 58 + normalized.len() + 80;
        if self.prefix.len() as u64 + required as u64 > MAX_HISTORY {
            return Err(LogError::Limit);
        }
        let binding_offset = self.prefix.len() as u64;
        let start = Instant::now();
        let written = self.write_transaction(&txn, &normalized, event_id, sequence, &frames);
        self.append_time += start.elapsed();
        let time = match written {
            Ok(time) => time,
            Err((boundary, e)) => {
                self.poisoned = true;
                return Ok(if boundary {
                    Outcome::Indeterminate {
                        request_id: txn.request_id,
                    }
                } else {
                    Outcome::Rejected {
                        reason: Rejection::Io(e.to_string()),
                    }
                });
            }
        };
        let outcome = Outcome::Committed {
            sequence,
            physical_ordinal: last_ordinal,
            durability_time: time,
            achieved: txn.durability,
        };
        self.bindings.insert(
            txn.request_id.0,
            Binding {
                event: event_id,
                hash: sha256(&normalized),
                normalized,
                outcome: outcome.clone(),
                offset: binding_offset,
            },
        );
        self.next_sequence = next_sequence;
        self.ordinal = last_ordinal;
        self.position = Some(Position {
            sequence,
            physical_ordinal: last_ordinal,
            offset: self.prefix.len() as u64,
        });
        self.payload_bytes += txn.payload.len() as u64;
        self.state = Arc::new(next);
        Ok(outcome)
    }
    fn submit(&mut self, bytes: &[u8]) -> io::Result<()> {
        let n = self.writer.write(bytes)?;
        if n != bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                format!("short write {n}/{}", bytes.len()),
            ));
        }
        self.prefix.extend_from_slice(bytes);
        Ok(())
    }
    fn write_transaction(
        &mut self,
        txn: &Transaction,
        normalized: &[u8],
        event: Uuid,
        sequence: u64,
        frames: &[Vec<u8>],
    ) -> Result<i64, (bool, io::Error)> {
        for (frame, point) in
            frames
                .iter()
                .zip([Point::Binding, Point::Reservation, Point::Provisional])
        {
            self.submit(frame).map_err(|e| (false, e))?;
            if txn.durability == Durability::D2 {
                self.writer.synchronize().map_err(|e| (false, e))?;
            }
            (self.hook)(point);
        }
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| (false, io::Error::other(e)))?
            .as_nanos();
        let time = i64::try_from(time).map_err(|e| (false, io::Error::other(e)))?;
        let final_record = Record {
            physical_ordinal: self.ordinal + 4,
            integrity: IntegrityProfile::Crc32c,
            body: Body::Final {
                event_id: event,
                request_id: txn.request_id,
                sequence,
                durability_time: time,
                complete_envelope: envelope::complete(txn, normalized, event, sequence, time),
            },
        };
        let final_bytes = encode(&final_record)
            .map_err(|e| (false, io::Error::other(format!("final encode {e:?}"))))?;
        let crc = u32::from_le_bytes(final_bytes[28..32].try_into().expect("encoded RF1 header"));
        let commit = encode(&Record {
            physical_ordinal: self.ordinal + 5,
            integrity: IntegrityProfile::Crc32c,
            body: Body::Commit {
                event_id: event,
                sequence,
                final_ordinal: self.ordinal + 4,
                final_crc32c: crc,
                group_id: 0,
                member_index: 0,
                member_count: 1,
            },
        })
        .map_err(|e| (false, io::Error::other(format!("commit encode {e:?}"))))?;
        self.submit(&final_bytes).map_err(|e| (true, e))?;
        (self.hook)(Point::Final);
        self.submit(&commit).map_err(|e| (true, e))?;
        (self.hook)(Point::Commit);
        if txn.durability == Durability::D2 {
            self.writer.synchronize().map_err(|e| (true, e))?;
        }
        (self.hook)(Point::Synced);
        Ok(time)
    }
    pub fn checkpoint(
        &self,
        encoder: impl FnOnce(&S) -> Result<Vec<u8>, String>,
    ) -> Result<CheckpointRef, LogError> {
        if self.poisoned {
            return Err(LogError::Poisoned);
        }
        checkpoint::write(self, encoder)
    }
}
/// Local event/request namespaces; monotone physical ordinal gives unique v4-shaped IDs.
/// Experimental deterministic assignment, not a random UUID generator.
pub fn identity(n: u64, domain: u8) -> Uuid {
    let mut id = [0u8; 16];
    id[0] = domain;
    id[6] = 0x40;
    id[8] = 0x80;
    id[9..16].copy_from_slice(&n.to_le_bytes()[..7]);
    Uuid(id)
}
