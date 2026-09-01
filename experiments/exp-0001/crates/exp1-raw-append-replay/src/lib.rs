//! Bounded, process-local EXP1-B1-RF1 raw D1 submission and physical replay.
//!
//! Successful submission means only that every byte was accepted by ordinary
//! file writes. It makes no stable-storage, namespace, D2/D3, or canonical claim.

#![forbid(unsafe_code)]

pub mod mapping;
pub mod reference_context;

use exp1_record_format::{
    Error as FormatError, Record, ScanLimits, ScanTermination, decode, scan_with_limits,
};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct D1SubmissionReceipt {
    pub starting_offset: u64,
    pub byte_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendFailure {
    InvalidFrame(FormatError),
    OffsetOverflow,
    ZeroProgress,
    Io(io::ErrorKind),
    Poisoned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppendError {
    pub failure: AppendFailure,
    pub starting_offset: u64,
    pub submitted_bytes: usize,
}

/// A mutable, single-owner appender. It deliberately offers no synchronization operation.
pub struct RawAppender {
    file: File,
    next_offset: u64,
    poisoned: bool,
}

impl RawAppender {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let next_offset = file.metadata()?.len();
        Ok(Self {
            file,
            next_offset,
            poisoned: false,
        })
    }

    pub fn append(&mut self, frame: &[u8]) -> Result<D1SubmissionReceipt, AppendError> {
        append_to(
            &mut self.file,
            &mut self.next_offset,
            &mut self.poisoned,
            frame,
        )
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }
}

fn append_to<W: Write>(
    writer: &mut W,
    next_offset: &mut u64,
    poisoned: &mut bool,
    frame: &[u8],
) -> Result<D1SubmissionReceipt, AppendError> {
    let start = *next_offset;
    if *poisoned {
        return Err(AppendError {
            failure: AppendFailure::Poisoned,
            starting_offset: start,
            submitted_bytes: 0,
        });
    }
    if let Err(error) = decode(frame) {
        return Err(AppendError {
            failure: AppendFailure::InvalidFrame(error),
            starting_offset: start,
            submitted_bytes: 0,
        });
    }
    let Ok(length) = u64::try_from(frame.len()) else {
        *poisoned = true;
        return Err(AppendError {
            failure: AppendFailure::OffsetOverflow,
            starting_offset: start,
            submitted_bytes: 0,
        });
    };
    let Some(end) = start.checked_add(length) else {
        *poisoned = true;
        return Err(AppendError {
            failure: AppendFailure::OffsetOverflow,
            starting_offset: start,
            submitted_bytes: 0,
        });
    };
    let mut submitted = 0;
    while submitted < frame.len() {
        match writer.write(&frame[submitted..]) {
            Ok(0) => {
                *poisoned = true;
                return Err(AppendError {
                    failure: AppendFailure::ZeroProgress,
                    starting_offset: start,
                    submitted_bytes: submitted,
                });
            }
            Ok(count) => submitted += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => {
                *poisoned = true;
                return Err(AppendError {
                    failure: AppendFailure::Io(error.kind()),
                    starting_offset: start,
                    submitted_bytes: submitted,
                });
            }
        }
    }
    *next_offset = end;
    Ok(D1SubmissionReceipt {
        starting_offset: start,
        byte_count: submitted,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhysicalRecord {
    pub offset: u64,
    pub extent: usize,
    pub bytes: Vec<u8>,
    pub record: Record,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayTermination {
    CleanEof,
    TerminalTruncation {
        offset: u64,
    },
    Failure {
        offset: u64,
        error: FormatError,
    },
    /// A read failed after `offset` source bytes were safely buffered.
    ///
    /// This terminal condition takes deterministic precedence over a format
    /// failure or truncation within those buffered bytes. The buffered bytes
    /// are still scanned fail-closed, so the report retains only the fully
    /// validated physical prefix preceding either condition.
    IoFailure {
        offset: u64,
        error: io::ErrorKind,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayReport {
    pub accepted_prefix: Vec<u8>,
    pub records: Vec<PhysicalRecord>,
    pub scanned_bytes: u64,
    pub termination: ReplayTermination,
}

pub fn reopen_and_replay(path: impl AsRef<Path>, limits: ScanLimits) -> ReplayReport {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return io_failure(0, error.kind()),
    };
    replay_from_reader(&mut file, limits)
}

fn replay_from_reader<R: Read>(reader: &mut R, limits: ScanLimits) -> ReplayReport {
    let capacity = match limits
        .max_scan_bytes
        .checked_add(1)
        .and_then(|n| usize::try_from(n).ok())
    {
        Some(value) => value,
        None => {
            return ReplayReport {
                accepted_prefix: Vec::new(),
                records: Vec::new(),
                scanned_bytes: 0,
                termination: ReplayTermination::Failure {
                    offset: 0,
                    error: FormatError::ScanByteLimit,
                },
            };
        }
    };
    let mut bytes = Vec::new();
    let read_error = Read::by_ref(reader)
        .take(capacity as u64)
        .read_to_end(&mut bytes)
        .err()
        .map(|error| error.kind());
    report_from_buffer(bytes, limits, capacity, read_error)
}

fn report_from_buffer(
    bytes: Vec<u8>,
    limits: ScanLimits,
    capacity: usize,
    read_error: Option<io::ErrorKind>,
) -> ReplayReport {
    let outcome = scan_with_limits(&bytes, limits);
    let accepted_len = usize::try_from(outcome.scanned_bytes).unwrap_or(0);
    let accepted_prefix = bytes[..accepted_len].to_vec();
    let mut offset = 0usize;
    let mut records = Vec::with_capacity(outcome.records.len());
    for record in outcome.records {
        let extent = u32::from_le_bytes(
            bytes[offset + 8..offset + 12]
                .try_into()
                .expect("validated header"),
        ) as usize;
        records.push(PhysicalRecord {
            offset: offset as u64,
            extent,
            bytes: bytes[offset..offset + extent].to_vec(),
            record,
        });
        offset += extent;
    }
    let termination = if let Some(error) = read_error {
        ReplayTermination::IoFailure {
            offset: bytes.len() as u64,
            error,
        }
    } else {
        match outcome.termination {
            ScanTermination::CleanEof if bytes.len() == capacity => ReplayTermination::Failure {
                offset: outcome.scanned_bytes,
                error: FormatError::ScanByteLimit,
            },
            ScanTermination::CleanEof => ReplayTermination::CleanEof,
            ScanTermination::TerminalTruncation { offset } => {
                ReplayTermination::TerminalTruncation { offset }
            }
            ScanTermination::Failure { offset, error } => {
                ReplayTermination::Failure { offset, error }
            }
        }
    };
    ReplayReport {
        accepted_prefix,
        records,
        scanned_bytes: outcome.scanned_bytes,
        termination,
    }
}

fn io_failure(offset: u64, error: io::ErrorKind) -> ReplayReport {
    ReplayReport {
        accepted_prefix: Vec::new(),
        records: Vec::new(),
        scanned_bytes: offset,
        termination: ReplayTermination::IoFailure { offset, error },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct Scripted {
        actions: VecDeque<Result<usize, io::ErrorKind>>,
        bytes: Vec<u8>,
    }

    struct FailingReader {
        bytes: Vec<u8>,
        position: usize,
        fail_at: usize,
        error: io::ErrorKind,
    }
    impl Read for FailingReader {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.position == self.fail_at {
                return Err(io::Error::from(self.error));
            }
            let available = self.fail_at - self.position;
            let count = available.min(output.len());
            output[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
            self.position += count;
            Ok(count)
        }
    }
    impl Write for Scripted {
        fn write(&mut self, input: &[u8]) -> io::Result<usize> {
            match self.actions.pop_front().unwrap_or(Ok(input.len())) {
                Ok(n) => {
                    let n = n.min(input.len());
                    self.bytes.extend_from_slice(&input[..n]);
                    Ok(n)
                }
                Err(kind) => Err(io::Error::from(kind)),
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    fn v1() -> Vec<u8> {
        hex(
            "5244453101000100440000002400000001000000000000000000000000000000101112131415461798191a1b1c1d1e1f000102030405460788090a0b0c0d0e0f00000000",
        )
    }
    fn v3() -> Vec<u8> {
        hex(
            "5244453101000201500000003000000002000000000000000000000041f0e427101112131415461798191a1b1c1d1e1f000102030405460788090a0b0c0d0e0f07000000000000000700000000000000",
        )
    }
    fn hex(value: &str) -> Vec<u8> {
        (0..value.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn write_machine_handles_short_interrupted_and_poison_paths() {
        let frame = v1();
        let mut writer = Scripted {
            actions: [Err(io::ErrorKind::Interrupted), Ok(7), Ok(frame.len())].into(),
            bytes: vec![],
        };
        let (mut offset, mut poison) = (9, false);
        assert_eq!(
            append_to(&mut writer, &mut offset, &mut poison, &frame).unwrap(),
            D1SubmissionReceipt {
                starting_offset: 9,
                byte_count: frame.len()
            }
        );
        assert_eq!(writer.bytes, frame);
        assert_eq!(offset, 9 + frame.len() as u64);

        for (actions, failure, progress) in [
            (VecDeque::from([Ok(0)]), AppendFailure::ZeroProgress, 0),
            (
                VecDeque::from([Ok(3), Err(io::ErrorKind::Other)]),
                AppendFailure::Io(io::ErrorKind::Other),
                3,
            ),
            (
                VecDeque::from([Err(io::ErrorKind::OutOfMemory)]),
                AppendFailure::Io(io::ErrorKind::OutOfMemory),
                0,
            ),
        ] {
            let mut writer = Scripted {
                actions,
                bytes: vec![],
            };
            let (mut offset, mut poison) = (0, false);
            let error = append_to(&mut writer, &mut offset, &mut poison, &frame).unwrap_err();
            assert_eq!((error.failure, error.submitted_bytes), (failure, progress));
            assert!(poison);
            assert_eq!(
                append_to(&mut writer, &mut offset, &mut poison, &frame)
                    .unwrap_err()
                    .failure,
                AppendFailure::Poisoned
            );
        }
    }

    #[test]
    fn validation_precedes_writes_and_input_is_immutable() {
        let mut frame = v1();
        frame.push(0);
        let copy = frame.clone();
        let mut writer = Scripted {
            actions: VecDeque::new(),
            bytes: vec![],
        };
        let (mut offset, mut poison) = (0, false);
        assert!(matches!(
            append_to(&mut writer, &mut offset, &mut poison, &frame)
                .unwrap_err()
                .failure,
            AppendFailure::InvalidFrame(FormatError::TrailingBytes)
        ));
        assert_eq!(frame, copy);
        assert!(writer.bytes.is_empty());
        assert!(!poison);

        let mut writer = Scripted {
            actions: VecDeque::new(),
            bytes: vec![],
        };
        let (mut offset, mut poison) = (u64::MAX, false);
        let error = append_to(&mut writer, &mut offset, &mut poison, &v1()).unwrap_err();
        assert_eq!(error.failure, AppendFailure::OffsetOverflow);
        assert!(poison);
        assert!(writer.bytes.is_empty());
    }

    #[test]
    fn read_io_failure_retains_only_the_validated_buffered_prefix() {
        let first = v1();
        let second = v3();
        let adjacent = [first.as_slice(), second.as_slice()].concat();
        let damaged = [first.as_slice(), &[1; 32]].concat();

        for (bytes, fail_at, accepted_records, scanned_bytes) in [
            (adjacent.clone(), 0, 0, 0),
            (adjacent.clone(), first.len(), 1, first.len()),
            (adjacent.clone(), first.len() + 40, 1, first.len()),
            (damaged.clone(), damaged.len(), 1, first.len()),
        ] {
            let mut reader = FailingReader {
                bytes,
                position: 0,
                fail_at,
                error: io::ErrorKind::Other,
            };
            let report = replay_from_reader(&mut reader, ScanLimits::default());
            assert_eq!(report.records.len(), accepted_records);
            assert_eq!(report.scanned_bytes, scanned_bytes as u64);
            assert_eq!(report.accepted_prefix, first[..scanned_bytes].to_vec());
            assert_eq!(
                report.termination,
                ReplayTermination::IoFailure {
                    offset: fail_at as u64,
                    error: io::ErrorKind::Other,
                }
            );
            if accepted_records == 1 {
                assert_eq!(report.records[0].bytes, first);
                assert_eq!(
                    (report.records[0].offset, report.records[0].extent),
                    (0, 68)
                );
            }
        }
    }

    #[test]
    fn identical_scripted_read_failures_produce_identical_reports() {
        let first = v1();
        let second = v3();
        let bytes = [first.as_slice(), &second[..40]].concat();
        let scan = || {
            let mut reader = FailingReader {
                fail_at: bytes.len(),
                bytes: bytes.clone(),
                position: 0,
                error: io::ErrorKind::UnexpectedEof,
            };
            replay_from_reader(&mut reader, ScanLimits::default())
        };
        assert_eq!(scan(), scan());
    }
}
