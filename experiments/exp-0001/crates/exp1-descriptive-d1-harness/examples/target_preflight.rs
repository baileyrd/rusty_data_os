use exp1_descriptive_d1_harness::{
    ArtifactClassificationV1, RetentionOutcomeV1, TargetArchitectureV1, TargetPlatformV1,
    TargetPreflightCallDispositionV1, TargetPreflightRequest, TargetPreflightRetention,
    run_target_preflight,
};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const INVALID_DIAGNOSTIC: &str = "target_preflight: invalid arguments or request\n";
const READY_DIAGNOSTIC: &str = "target_preflight: preflight subset ready\n";
const NOT_READY_DIAGNOSTIC: &str = "target_preflight: preflight blocked or invalid\n";
const RETENTION_DIAGNOSTIC: &str = "target_preflight: artifact serialization or retention failed\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DispatchOutcome {
    ReadyRetained,
    NotReadyRetained,
    RequestInvalid,
    SerializationOrRetentionFailed,
}

fn request_is_valid(request: &TargetPreflightRequest<'_>) -> bool {
    let revision = request.repository_revision;
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return false;
    }

    let build = request.build_identity;
    if build.is_empty()
        || build.len() > 128
        || !build
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'/' && byte != b'\\')
    {
        return false;
    }

    let identity = request.measured_file_identity;
    !identity.is_empty()
        && identity.len() <= 64
        && identity.is_ascii()
        && identity
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && identity.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
        && !identity.contains("..")
        && !identity.contains("://")
        && !request
            .measured_file_path
            .components()
            .any(|component| component.as_os_str() == OsStr::new(identity))
}

fn live_dispatch(
    request: &TargetPreflightRequest<'_>,
    retention: &mut dyn TargetPreflightRetention,
) -> DispatchOutcome {
    let execution = run_target_preflight(request, retention);
    match execution.disposition {
        TargetPreflightCallDispositionV1::RequestInvalid => DispatchOutcome::RequestInvalid,
        TargetPreflightCallDispositionV1::SerializationFailed
        | TargetPreflightCallDispositionV1::RetentionFailed => {
            DispatchOutcome::SerializationOrRetentionFailed
        }
        TargetPreflightCallDispositionV1::Completed => {
            if !matches!(execution.retention, RetentionOutcomeV1::Success { .. }) {
                return DispatchOutcome::SerializationOrRetentionFailed;
            }
            match execution.artifact.map(|artifact| artifact.classification) {
                Some(ArtifactClassificationV1::PreflightSubsetReady) => {
                    DispatchOutcome::ReadyRetained
                }
                Some(ArtifactClassificationV1::Blocked | ArtifactClassificationV1::Invalid) => {
                    DispatchOutcome::NotReadyRetained
                }
                None => DispatchOutcome::SerializationOrRetentionFailed,
            }
        }
    }
}

fn run_with<D, O, E>(
    arguments: Vec<OsString>,
    stdout: &mut O,
    stderr: &mut E,
    mut dispatch: D,
) -> u8
where
    D: FnMut(&TargetPreflightRequest<'_>, &mut dyn TargetPreflightRetention) -> DispatchOutcome,
    O: Write,
    E: Write,
{
    let [revision, build, identity, path]: [OsString; 4] = match arguments.try_into() {
        Ok(arguments) => arguments,
        Err(_) => {
            let _ = stderr.write_all(INVALID_DIAGNOSTIC.as_bytes());
            return 64;
        }
    };
    let (Ok(revision), Ok(build), Ok(identity)) = (
        revision.into_string(),
        build.into_string(),
        identity.into_string(),
    ) else {
        let _ = stderr.write_all(INVALID_DIAGNOSTIC.as_bytes());
        return 64;
    };
    let path = PathBuf::from(path);
    let request = TargetPreflightRequest {
        repository_revision: &revision,
        build_identity: &build,
        expected_platform: TargetPlatformV1::Fedora44Linux,
        expected_architecture: TargetArchitectureV1::X86_64,
        measured_file_path: &path,
        measured_file_identity: &identity,
    };
    if !request_is_valid(&request) {
        let _ = stderr.write_all(INVALID_DIAGNOSTIC.as_bytes());
        return 64;
    }

    let (status, diagnostic) = match dispatch(&request, stdout) {
        DispatchOutcome::ReadyRetained => (0, READY_DIAGNOSTIC),
        DispatchOutcome::NotReadyRetained => (2, NOT_READY_DIAGNOSTIC),
        DispatchOutcome::RequestInvalid => (64, INVALID_DIAGNOSTIC),
        DispatchOutcome::SerializationOrRetentionFailed => (70, RETENTION_DIAGNOSTIC),
    };
    let _ = stderr.write_all(diagnostic.as_bytes());
    status
}

fn main() -> ExitCode {
    let arguments = std::env::args_os().skip(1).collect();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    ExitCode::from(run_with(arguments, &mut stdout, &mut stderr, live_dispatch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::path::Path;

    const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";

    fn arguments(revision: &str, build: &str, identity: &str, path: &Path) -> Vec<OsString> {
        vec![
            revision.into(),
            build.into(),
            identity.into(),
            path.as_os_str().to_owned(),
        ]
    }

    fn valid_arguments() -> Vec<OsString> {
        arguments(
            REVISION,
            "release-build",
            "measured-alpha",
            Path::new("/tmp/input"),
        )
    }

    #[test]
    fn exact_argument_count_is_required_before_dispatch() {
        for supplied in [vec![], vec![OsString::from(REVISION)], {
            let mut values = valid_arguments();
            values.push("extra".into());
            values
        }] {
            let calls = Cell::new(0);
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                run_with(supplied, &mut stdout, &mut stderr, |_, _| {
                    calls.set(calls.get() + 1);
                    DispatchOutcome::ReadyRetained
                }),
                64
            );
            assert_eq!(calls.get(), 0);
            assert!(stdout.is_empty());
            assert_eq!(stderr, INVALID_DIAGNOSTIC.as_bytes());
        }
    }

    #[test]
    fn malformed_and_boundary_inputs_are_validated_before_dispatch() {
        let build_128 = "b".repeat(128);
        let identity_64 = format!("a{}", "b".repeat(63));
        let valid = [
            arguments(REVISION, "b", "a", Path::new("/tmp/input")),
            arguments(REVISION, &build_128, &identity_64, Path::new("/tmp/input")),
        ];
        for supplied in valid {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                run_with(supplied, &mut stdout, &mut stderr, |_, _| {
                    DispatchOutcome::ReadyRetained
                }),
                0
            );
        }

        let invalid = [
            arguments(&"a".repeat(39), "b", "identity", Path::new("/tmp/input")),
            arguments(&"A".repeat(40), "b", "identity", Path::new("/tmp/input")),
            arguments(REVISION, "", "identity", Path::new("/tmp/input")),
            arguments(
                REVISION,
                &"b".repeat(129),
                "identity",
                Path::new("/tmp/input"),
            ),
            arguments(REVISION, "bad/build", "identity", Path::new("/tmp/input")),
            arguments(REVISION, "b", "Upper", Path::new("/tmp/input")),
            arguments(REVISION, "b", "a..b", Path::new("/tmp/input")),
            arguments(
                REVISION,
                "b",
                &format!("a{}", "b".repeat(64)),
                Path::new("/tmp/input"),
            ),
            arguments(REVISION, "b", "secret", Path::new("/tmp/secret/input")),
        ];
        for supplied in invalid {
            let calls = Cell::new(0);
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                run_with(supplied, &mut stdout, &mut stderr, |_, _| {
                    calls.set(calls.get() + 1);
                    DispatchOutcome::ReadyRetained
                }),
                64
            );
            assert_eq!(calls.get(), 0);
        }
    }

    #[cfg(unix)]
    #[test]
    fn invalid_text_is_rejected_without_leaking_supplied_bytes() {
        use std::os::unix::ffi::OsStringExt;

        for index in 0..3 {
            let mut supplied = valid_arguments();
            supplied[index] = OsString::from_vec(vec![b's', 0xff, b'e']);
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                run_with(supplied, &mut stdout, &mut stderr, |_, _| panic!(
                    "dispatched"
                )),
                64
            );
            assert!(stdout.is_empty());
            assert_eq!(stderr, INVALID_DIAGNOSTIC.as_bytes());
            assert!(!stderr.contains(&0xff));
        }
    }

    #[test]
    fn valid_input_dispatches_once_with_exact_request() {
        let calls = Cell::new(0);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let status = run_with(valid_arguments(), &mut stdout, &mut stderr, |request, _| {
            calls.set(calls.get() + 1);
            assert_eq!(request.repository_revision, REVISION);
            assert_eq!(request.build_identity, "release-build");
            assert_eq!(request.measured_file_identity, "measured-alpha");
            assert_eq!(request.measured_file_path, Path::new("/tmp/input"));
            assert_eq!(request.expected_platform, TargetPlatformV1::Fedora44Linux);
            assert_eq!(request.expected_architecture, TargetArchitectureV1::X86_64);
            DispatchOutcome::ReadyRetained
        });
        assert_eq!(status, 0);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn every_dispatch_outcome_has_the_frozen_exit_and_diagnostic() {
        for (outcome, status, diagnostic) in [
            (DispatchOutcome::ReadyRetained, 0, READY_DIAGNOSTIC),
            (DispatchOutcome::NotReadyRetained, 2, NOT_READY_DIAGNOSTIC),
            (DispatchOutcome::RequestInvalid, 64, INVALID_DIAGNOSTIC),
            (
                DispatchOutcome::SerializationOrRetentionFailed,
                70,
                RETENTION_DIAGNOSTIC,
            ),
        ] {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            assert_eq!(
                run_with(valid_arguments(), &mut stdout, &mut stderr, |_, _| outcome),
                status
            );
            assert!(stdout.is_empty());
            assert_eq!(stderr, diagnostic.as_bytes());
        }
    }

    struct FailingSink {
        bytes: Vec<u8>,
        fail_write: bool,
        fail_flush: bool,
    }

    impl Write for FailingSink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail_write {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "synthetic"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.fail_flush {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "synthetic"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn artifact_bytes_pass_directly_through_sink_and_io_failure_wins() {
        const ARTIFACT: &[u8] = b"{\"classification\":\"preflight_subset_ready\"}\n";
        for (fail_write, fail_flush) in [(false, false), (true, false), (false, true)] {
            let mut stdout = FailingSink {
                bytes: Vec::new(),
                fail_write,
                fail_flush,
            };
            let mut stderr = Vec::new();
            let status = run_with(valid_arguments(), &mut stdout, &mut stderr, |_, sink| {
                if sink.write_all(ARTIFACT).is_err() || sink.flush().is_err() {
                    DispatchOutcome::SerializationOrRetentionFailed
                } else {
                    DispatchOutcome::ReadyRetained
                }
            });
            assert_eq!(stdout.bytes, if fail_write { &[] } else { ARTIFACT });
            assert_eq!(status, if fail_write || fail_flush { 70 } else { 0 });
            assert_eq!(
                stderr,
                if fail_write || fail_flush {
                    RETENTION_DIAGNOSTIC
                } else {
                    READY_DIAGNOSTIC
                }
                .as_bytes()
            );
        }
    }

    #[test]
    fn diagnostics_never_contain_supplied_values_or_path() {
        let supplied = arguments(
            REVISION,
            "private-build",
            "private-id",
            Path::new("/private/path-value"),
        );
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            run_with(supplied, &mut stdout, &mut stderr, |_, _| {
                DispatchOutcome::NotReadyRetained
            }),
            2
        );
        let diagnostic = String::from_utf8(stderr).unwrap();
        for secret in [
            REVISION,
            "private-build",
            "private-id",
            "/private/path-value",
        ] {
            assert!(!diagnostic.contains(secret));
        }
    }
}
