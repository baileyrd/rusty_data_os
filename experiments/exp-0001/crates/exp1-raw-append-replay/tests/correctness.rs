use exp1_raw_append_replay::{RawAppender, ReplayTermination, reopen_and_replay};
use exp1_record_format::{Error, ScanLimits, crc32c};
use std::fs;
use std::path::PathBuf;

const V1: &str = "5244453101000100440000002400000001000000000000000000000000000000101112131415461798191a1b1c1d1e1f000102030405460788090a0b0c0d0e0f00000000";
const V2: &str = "52444531010003004c0000002c00000003000000000000000000000000000000000102030405460788090a0b0c0d0e0f07000000000000000000000000000000000001000400000044415441";
const V3: &str = "5244453101000201500000003000000002000000000000000000000041f0e427101112131415461798191a1b1c1d1e1f000102030405460788090a0b0c0d0e0f07000000000000000700000000000000";
const V4: &str = "524445310100050158000000380000000500000000000000000000008a1267a2000102030405460788090a0b0c0d0e0f101112131415461798191a1b1c1d1e1f070000000000000015cd5b07000000000400000044415441";

fn hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}
fn path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "exp1-raw-append-replay-{}-{tag}",
        std::process::id()
    ))
}
fn with_artifact(tag: &str, bytes: &[u8], check: impl FnOnce(&PathBuf)) {
    let path = path(tag);
    let _ = fs::remove_file(&path);
    fs::write(&path, bytes).unwrap();
    check(&path);
    fs::remove_file(path).unwrap();
}

#[test]
fn empty_and_stable_vectors_replay_with_exact_physical_metadata() {
    with_artifact("empty", &[], |path| {
        let report = reopen_and_replay(path, ScanLimits::default());
        assert_eq!(report.termination, ReplayTermination::CleanEof);
        assert!(report.records.is_empty());
        assert!(report.accepted_prefix.is_empty());
    });
    let v1 = hex(V1);
    let v3 = hex(V3);
    let artifact = [v1.as_slice(), v3.as_slice()].concat();
    with_artifact("adjacent", &artifact, |path| {
        let first = reopen_and_replay(path, ScanLimits::default());
        let second = reopen_and_replay(path, ScanLimits::default());
        assert_eq!(first, second);
        assert_eq!(first.termination, ReplayTermination::CleanEof);
        assert_eq!(first.accepted_prefix, artifact);
        assert_eq!((first.records[0].offset, first.records[0].extent), (0, 68));
        assert_eq!((first.records[1].offset, first.records[1].extent), (68, 80));
        assert_eq!(first.records[0].bytes, v1);
        assert_eq!(first.records[1].bytes, v3);
        assert_eq!(first.records[0].record.physical_ordinal, 1);
        assert_eq!(first.records[1].record.physical_ordinal, 2);
    });
}

#[test]
fn create_append_close_reopen_is_ordered_and_byte_identical() {
    let path = path("roundtrip");
    let _ = fs::remove_file(&path);
    let first = hex(V1);
    let second = hex(V3);
    {
        let mut appender = RawAppender::open(&path).unwrap();
        assert_eq!(appender.append(&first).unwrap().starting_offset, 0);
        let receipt = appender.append(&second).unwrap();
        assert_eq!((receipt.starting_offset, receipt.byte_count), (68, 80));
    }
    let report = reopen_and_replay(&path, ScanLimits::default());
    assert_eq!(report.accepted_prefix, [first, second].concat());
    assert_eq!(report.termination, ReplayTermination::CleanEof);
    fs::remove_file(path).unwrap();
}

#[test]
fn profile_zero_and_crc_check_vector_are_accepted() {
    assert_eq!(crc32c(b"123456789"), 0xe306_9283);
    let v2 = hex(V2);
    with_artifact("profile0", &v2, |path| {
        assert_eq!(
            reopen_and_replay(path, ScanLimits::default()).records.len(),
            1
        )
    });
    let v4 = hex(V4);
    let path = path("crc-append");
    let _ = fs::remove_file(&path);
    assert_eq!(
        RawAppender::open(&path)
            .unwrap()
            .append(&v4)
            .unwrap()
            .byte_count,
        88
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn every_header_and_body_truncation_boundary_is_terminal() {
    let frame = hex(V1);
    for end in 1..frame.len() {
        with_artifact(&format!("trunc-{end}"), &frame[..end], |path| {
            let report = reopen_and_replay(path, ScanLimits::default());
            assert_eq!(
                report.termination,
                ReplayTermination::TerminalTruncation { offset: 0 },
                "boundary {end}"
            );
            assert!(report.records.is_empty());
            assert!(report.accepted_prefix.is_empty());
        });
    }
    let next = hex(V3);
    let artifact = [frame.as_slice(), &next[..40]].concat();
    with_artifact("prefix-trunc", &artifact, |path| {
        let report = reopen_and_replay(path, ScanLimits::default());
        assert_eq!(
            report.termination,
            ReplayTermination::TerminalTruncation { offset: 68 }
        );
        assert_eq!(report.accepted_prefix, frame);
    });
}

#[test]
fn malformed_unknown_crc_garbage_order_and_ambiguous_damage_fail_closed() {
    let base = hex(V1);
    let mut cases: Vec<(Vec<u8>, Error)> = Vec::new();
    let mut malformed = base.clone();
    malformed[8..12].copy_from_slice(&31_u32.to_le_bytes());
    cases.push((malformed, Error::InvalidLength));
    let mut excessive = base.clone();
    excessive[8..12].copy_from_slice(&16_777_217_u32.to_le_bytes());
    cases.push((excessive, Error::Oversize));
    let mut unknown = base.clone();
    unknown[6] = 7;
    cases.push((unknown, Error::UnknownKind));
    let mut bad_profile = base.clone();
    bad_profile[7] = 2;
    cases.push((bad_profile, Error::UnsupportedIntegrity));
    let mut crc = hex(V4);
    *crc.last_mut().unwrap() ^= 1;
    cases.push((crc, Error::CrcMismatch));
    let mut garbage = base.clone();
    garbage.extend_from_slice(&[1; 32]);
    cases.push((garbage, Error::BadMagic));
    let mut duplicate = base.clone();
    duplicate.extend_from_slice(&base);
    cases.push((duplicate, Error::OrdinalOrder));
    for (index, (bytes, expected)) in cases.into_iter().enumerate() {
        with_artifact(&format!("failure-{index}"), &bytes, |path| {
            assert!(
                matches!(reopen_and_replay(path, ScanLimits::default()).termination, ReplayTermination::Failure { error, .. } if error == expected)
            );
        });
    }
    let v4 = hex(V4);
    let ambiguous = [base.as_slice(), &v4[..40], b"RDE1"].concat();
    with_artifact("ambiguous", &ambiguous, |path| {
        assert!(matches!(
            reopen_and_replay(path, ScanLimits::default()).termination,
            ReplayTermination::Failure {
                error: Error::InteriorDamage,
                ..
            }
        ))
    });
}

#[test]
fn all_resource_limits_and_open_io_failure_are_reported() {
    let first = hex(V1);
    let second = hex(V3);
    let artifact = [first.as_slice(), second.as_slice()].concat();
    for (index, (limits, error)) in [
        (
            ScanLimits {
                max_records: 1,
                ..ScanLimits::default()
            },
            Error::RecordLimit,
        ),
        (
            ScanLimits {
                max_record_len: 67,
                ..ScanLimits::default()
            },
            Error::Oversize,
        ),
        (
            ScanLimits {
                max_scan_bytes: 100,
                ..ScanLimits::default()
            },
            Error::ScanByteLimit,
        ),
        (
            ScanLimits {
                max_diagnostic_bytes: 68,
                ..ScanLimits::default()
            },
            Error::DiagnosticLimit,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        with_artifact(&format!("limit-{index}"), &artifact, |path| {
            assert!(
                matches!(reopen_and_replay(path, limits).termination, ReplayTermination::Failure { error: actual, .. } if actual == error)
            )
        });
    }
    let missing = path("missing");
    let _ = fs::remove_file(&missing);
    assert!(matches!(
        reopen_and_replay(missing, ScanLimits::default()).termination,
        ReplayTermination::IoFailure { .. }
    ));
}
