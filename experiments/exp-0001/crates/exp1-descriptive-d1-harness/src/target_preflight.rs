//! R33's explicit target-preflight boundary and deterministic JSON-lines artifact.
//!
//! The public entry point is live-call-capable but is never called by this crate's tests.

use crate::LiveCaptureBoundary;
use crate::linux_capture::{
    self, Clock, ErrorReason, FileLength, FileLengthSource, Outcome, OverflowReason, ParseReason,
    PerfCleanupState, PerfCounter, PerfEvent, ProcessIo, ResourceUsage, Statm, StatusMemory,
    UnavailableReason,
};
use crate::orchestration::{CaptureBoundary, MeasuredFileReference};
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub const TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2: &str = "EXP-0001-R34/target-preflight-artifact-v2";
pub const TRACEFS_REASON_V1: &str = "R33 target preflight deliberately did not invoke tracefs";

pub type UnavailableReasonV1 = UnavailableReason;
pub type OverflowReasonV1 = OverflowReason;
pub type ErrorReasonV1 = ErrorReason;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetPlatformV1 {
    Fedora44Linux,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetArchitectureV1 {
    X86_64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchitectureObservationV2 {
    X86_64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformDispositionV2 {
    ProspectiveFedora44Linux,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactClassificationV1 {
    PreflightSubsetReady,
    Blocked,
    Invalid,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetPreflightCallDispositionV1 {
    RequestInvalid,
    Completed,
    SerializationFailed,
    RetentionFailed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestFailureReasonV1 {
    InvalidRepositoryRevision,
    InvalidBuildIdentity,
    InvalidMeasuredFileIdentity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionOperationV1 {
    WriteAll,
    Flush,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotAttemptedReasonV1 {
    RequestInvalid(RequestFailureReasonV1),
    SerializationFailure,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IoErrorKindV1 {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    InProgress,
    Other,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionIoErrorV1 {
    pub kind: IoErrorKindV1,
    pub raw_os_error: Option<i32>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionOutcomeV1 {
    NotAttempted {
        reason: NotAttemptedReasonV1,
    },
    Success {
        serialized_byte_length: u64,
    },
    IoFailure {
        operation: RetentionOperationV1,
        error: RetentionIoErrorV1,
    },
}

pub trait TargetPreflightRetention {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
}
impl<T: Write> TargetPreflightRetention for T {
    fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        Write::write_all(self, bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        Write::flush(self)
    }
}

#[derive(Clone, Debug)]
pub struct TargetPreflightRequest<'a> {
    pub repository_revision: &'a str,
    pub build_identity: &'a str,
    pub expected_platform: TargetPlatformV1,
    pub expected_architecture: TargetArchitectureV1,
    pub measured_file_path: &'a Path,
    pub measured_file_identity: &'a str,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetPlatformV2 {
    pub expected: TargetPlatformV1,
    pub disposition: PlatformDispositionV2,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetArchitectureV2 {
    pub expected: TargetArchitectureV1,
    pub observed: ArchitectureObservationV2,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitV1 {
    Nanoseconds,
    Bytes,
    StatmPages,
    ResourceUsage,
    ProcessIo,
    PerfCounter,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeV1 {
    Observation,
    Process,
    MeasuredThread,
    MeasuredFile,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreflightOutcomeV1<T> {
    Outcome(Outcome<T>),
    NotAttempted { failure_id: String },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceV1<T> {
    pub identity: SourceIdentityV1,
    pub scope: ScopeV1,
    pub unit: UnitV1,
    pub outcome: PreflightOutcomeV1<T>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceIdentityV1 {
    ClockResolutionRealtime,
    ClockResolutionMonotonicRaw,
    Realtime,
    MonotonicRaw,
    ProcessRusage,
    ThreadRusage,
    Statm,
    Status,
    ProcessIo,
    FileLength,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeasuredFileV1 {
    pub identity: String,
    pub open: PreflightOutcomeV1<()>,
    pub regular_file: PreflightOutcomeV1<bool>,
    pub length: SourceV1<FileLength>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourcesV1 {
    pub clock_resolution_realtime: SourceV1<i128>,
    pub clock_resolution_monotonic_raw: SourceV1<i128>,
    pub realtime: SourceV1<i128>,
    pub monotonic_raw: SourceV1<i128>,
    pub process_rusage: SourceV1<ResourceUsage>,
    pub thread_rusage: SourceV1<ResourceUsage>,
    pub statm: SourceV1<Statm>,
    pub status: SourceV1<StatusMemory>,
    pub process_io: SourceV1<ProcessIo>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerfEventV1 {
    pub event: PerfEvent,
    pub scope: ScopeV1,
    pub unit: UnitV1,
    pub open: PreflightOutcomeV1<()>,
    pub stop_read: PreflightOutcomeV1<PerfCounter>,
    pub cleanup: PreflightOutcomeV1<()>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecyclePhaseV1 {
    RequestValidated,
    FileOpened,
    SourcesChecked,
    PerfChecked,
    OwnershipReleased,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasuredFileOwnershipReleaseV2 {
    NotAcquired,
    DropCompleted,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetPreflightLifecycleV2 {
    pub phases: Vec<LifecyclePhaseV1>,
    pub measured_file_ownership_release: MeasuredFileOwnershipReleaseV2,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailurePhaseV1 {
    PlatformValidation,
    ArchitectureValidation,
    MeasuredFileOpen,
    MeasuredFileRegularFile,
    ClockResolutionRealtime,
    ClockResolutionMonotonicRaw,
    SourceCapture,
    PerfOpen,
    PerfStopRead,
    PerfCleanup,
    OwnershipRelease,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureSourceV1 {
    Platform,
    Architecture,
    MeasuredFile,
    Source(SourceIdentityV1),
    Perf(PerfEvent),
    Lifecycle,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidStateReasonV1 {
    PlatformMismatch,
    ArchitectureMismatch,
    NotRegularFile,
    LifecycleViolation,
    OwnershipReleaseIncomplete,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FailureDetailV1 {
    Unavailable(UnavailableReasonV1),
    Permission(i32),
    Overflow(OverflowReasonV1),
    Error(ErrorReasonV1),
    InvalidState(InvalidStateReasonV1),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureObjectV1 {
    pub id: String,
    pub phase: FailurePhaseV1,
    pub source: FailureSourceV1,
    pub detail: FailureDetailV1,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracefsV1 {
    pub state: TracefsStateV1,
    pub reason: &'static str,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TracefsStateV1 {
    NotCollected,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetPreflightArtifactV2 {
    pub schema: &'static str,
    pub repository_revision: String,
    pub build_identity: String,
    pub platform: TargetPlatformV2,
    pub architecture: TargetArchitectureV2,
    pub measured_file: MeasuredFileV1,
    pub sources: SourcesV1,
    pub perf_events: [PerfEventV1; 4],
    pub lifecycle: TargetPreflightLifecycleV2,
    pub first_causal_failure: Option<FailureObjectV1>,
    pub cleanup_failures: Vec<FailureObjectV1>,
    pub tracefs: TracefsV1,
    pub classification: ArtifactClassificationV1,
    pub reasons: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetPreflightExecutionV2 {
    pub artifact: Option<TargetPreflightArtifactV2>,
    pub serialized_bytes: Option<Vec<u8>>,
    pub retention: RetentionOutcomeV1,
    pub disposition: TargetPreflightCallDispositionV1,
}

fn invalid(reason: RequestFailureReasonV1) -> TargetPreflightExecutionV2 {
    TargetPreflightExecutionV2 {
        artifact: None,
        serialized_bytes: None,
        retention: RetentionOutcomeV1::NotAttempted {
            reason: NotAttemptedReasonV1::RequestInvalid(reason),
        },
        disposition: TargetPreflightCallDispositionV1::RequestInvalid,
    }
}
fn validate(r: &TargetPreflightRequest<'_>) -> Result<(), RequestFailureReasonV1> {
    if r.repository_revision.len() != 40
        || !r
            .repository_revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(RequestFailureReasonV1::InvalidRepositoryRevision);
    }
    if r.build_identity.is_empty()
        || r.build_identity.len() > 128
        || !r
            .build_identity
            .bytes()
            .all(|b| b.is_ascii_graphic() && b != b'/' && b != b'\\')
    {
        return Err(RequestFailureReasonV1::InvalidBuildIdentity);
    }
    let id = r.measured_file_identity;
    if id.is_empty()
        || id.len() > 64
        || !id.is_ascii()
        || !id
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        || id.contains("..")
        || id.contains("://")
        || r.measured_file_path
            .components()
            .any(|c| c.as_os_str().to_str() == Some(id))
    {
        return Err(RequestFailureReasonV1::InvalidMeasuredFileIdentity);
    }
    Ok(())
}

trait ExecutionBoundary: CaptureBoundary {
    fn resolution(&mut self, clock: Clock) -> Outcome<i128>;
}
struct LiveBoundary<'a>(LiveCaptureBoundary<'a>);
impl CaptureBoundary for LiveBoundary<'_> {
    type PerfOwner = <LiveCaptureBoundary<'static> as CaptureBoundary>::PerfOwner;
    fn realtime(&mut self) -> Outcome<i128> {
        self.0.realtime()
    }
    fn monotonic_raw(&mut self) -> Outcome<i128> {
        self.0.monotonic_raw()
    }
    fn process_rusage(&mut self) -> Outcome<ResourceUsage> {
        self.0.process_rusage()
    }
    fn thread_rusage(&mut self) -> Outcome<ResourceUsage> {
        self.0.thread_rusage()
    }
    fn statm(&mut self) -> Outcome<Statm> {
        self.0.statm()
    }
    fn status(&mut self) -> Outcome<StatusMemory> {
        self.0.status()
    }
    fn process_io(&mut self) -> Outcome<ProcessIo> {
        self.0.process_io()
    }
    fn file_length(&mut self, f: MeasuredFileReference<'_>) -> Outcome<FileLength> {
        self.0.file_length(f)
    }
    fn open_perf(&mut self, e: PerfEvent) -> Outcome<Self::PerfOwner> {
        self.0.open_perf(e)
    }
    fn stop_perf(&mut self, o: &mut Self::PerfOwner, e: PerfEvent) -> Outcome<PerfCounter> {
        self.0.stop_perf(o, e)
    }
    fn cleanup_perf(&mut self, o: Self::PerfOwner, e: PerfEvent) -> Outcome<()> {
        self.0.cleanup_perf(o, e)
    }
}
impl ExecutionBoundary for LiveBoundary<'_> {
    fn resolution(&mut self, c: Clock) -> Outcome<i128> {
        linux_capture::clock_resolution(c)
    }
}

pub fn run_target_preflight(
    request: &TargetPreflightRequest<'_>,
    retention: &mut dyn TargetPreflightRetention,
) -> TargetPreflightExecutionV2 {
    if let Err(e) = validate(request) {
        return invalid(e);
    }
    let opened = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .create_new(false)
        .truncate(false)
        .append(false)
        .open(request.measured_file_path);
    let file = match opened {
        Ok(f) => f,
        Err(e) => return finish_artifact(build_open_failure(request, e), retention),
    };
    let regular = match file.metadata() {
        Ok(m) => m.file_type().is_file(),
        Err(e) => {
            drop(file);
            return finish_artifact(
                mark_drop_completed(build_metadata_failure(request, e)),
                retention,
            );
        }
    };
    if !regular {
        drop(file);
        return finish_artifact(mark_drop_completed(build_nonregular(request)), retention);
    }
    let cleanup = PerfCleanupState::default();
    let mut boundary = LiveBoundary(LiveCaptureBoundary::new(&cleanup));
    let artifact = execute_with_boundary(request, &file, &mut boundary);
    drop(file);
    finish_artifact(mark_drop_completed(artifact), retention)
}

fn base_source<T>(
    identity: SourceIdentityV1,
    scope: ScopeV1,
    unit: UnitV1,
    outcome: PreflightOutcomeV1<T>,
) -> SourceV1<T> {
    SourceV1 {
        identity,
        scope,
        unit,
        outcome,
    }
}
fn skipped<T>(id: &str) -> PreflightOutcomeV1<T> {
    PreflightOutcomeV1::NotAttempted {
        failure_id: id.to_owned(),
    }
}
fn failure_detail<T>(o: &Outcome<T>) -> Option<FailureDetailV1> {
    match o {
        Outcome::Success(_) => None,
        Outcome::Unavailable(v) => Some(FailureDetailV1::Unavailable(*v)),
        Outcome::Permission(v) => Some(FailureDetailV1::Permission(*v)),
        Outcome::Overflow(v) => Some(FailureDetailV1::Overflow(*v)),
        Outcome::Error(v) => Some(FailureDetailV1::Error(v.clone())),
    }
}
fn execute_with_boundary<B: ExecutionBoundary>(
    r: &TargetPreflightRequest<'_>,
    file: &impl std::os::fd::AsRawFd,
    b: &mut B,
) -> TargetPreflightArtifactV2 {
    let mut first: Option<FailureObjectV1> = None;
    macro_rules! call {
        ($phase:expr,$source:expr,$expr:expr) => {{
            if let Some(existing) = &first {
                skipped(existing.id.as_str())
            } else {
                let o = $expr;
                if let Some(d) = failure_detail(&o) {
                    let id = "failure-0001".to_owned();
                    first = Some(FailureObjectV1 {
                        id: id.clone(),
                        phase: $phase,
                        source: $source,
                        detail: d,
                    });
                }
                PreflightOutcomeV1::Outcome(o)
            }
        }};
    }
    let rr = call!(
        FailurePhaseV1::ClockResolutionRealtime,
        FailureSourceV1::Source(SourceIdentityV1::ClockResolutionRealtime),
        b.resolution(Clock::Realtime)
    );
    let rm = call!(
        FailurePhaseV1::ClockResolutionMonotonicRaw,
        FailureSourceV1::Source(SourceIdentityV1::ClockResolutionMonotonicRaw),
        b.resolution(Clock::MonotonicRaw)
    );
    let realtime = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::Realtime),
        b.realtime()
    );
    let mono = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::MonotonicRaw),
        b.monotonic_raw()
    );
    let pr = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::ProcessRusage),
        b.process_rusage()
    );
    let tr = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::ThreadRusage),
        b.thread_rusage()
    );
    let statm = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::Statm),
        b.statm()
    );
    let status = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::Status),
        b.status()
    );
    let pio = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::ProcessIo),
        b.process_io()
    );
    let fl = call!(
        FailurePhaseV1::SourceCapture,
        FailureSourceV1::Source(SourceIdentityV1::FileLength),
        b.file_length(MeasuredFileReference::borrowed(
            r.measured_file_identity,
            file
        ))
    );
    let mut cleanup_failures = Vec::new();
    let events = [
        PerfEvent::CpuCycles,
        PerfEvent::Instructions,
        PerfEvent::PageFaults,
        PerfEvent::ContextSwitches,
    ];
    let perf_events = events.map(|event| {
        if let Some(existing) = &first {
            let id = existing.id.clone();
            return PerfEventV1 {
                event,
                scope: ScopeV1::MeasuredThread,
                unit: UnitV1::PerfCounter,
                open: skipped(&id),
                stop_read: skipped(&id),
                cleanup: skipped(&id),
            };
        }
        let open = b.open_perf(event);
        match open {
            Outcome::Success(mut owner) => {
                let stop = b.stop_perf(&mut owner, event);
                if let Some(d) = failure_detail(&stop) {
                    let id = "failure-0001".to_owned();
                    first = Some(FailureObjectV1 {
                        id,
                        phase: FailurePhaseV1::PerfStopRead,
                        source: FailureSourceV1::Perf(event),
                        detail: d,
                    });
                }
                let clean = b.cleanup_perf(owner, event);
                let clean_out = match clean {
                    Outcome::Success(()) => PreflightOutcomeV1::Outcome(Outcome::Success(())),
                    bad => {
                        cleanup_failures.push(FailureObjectV1 {
                            id: String::new(),
                            phase: FailurePhaseV1::PerfCleanup,
                            source: FailureSourceV1::Perf(event),
                            detail: failure_detail(&bad).unwrap(),
                        });
                        PreflightOutcomeV1::Outcome(bad)
                    }
                };
                PerfEventV1 {
                    event,
                    scope: ScopeV1::MeasuredThread,
                    unit: UnitV1::PerfCounter,
                    open: PreflightOutcomeV1::Outcome(Outcome::Success(())),
                    stop_read: PreflightOutcomeV1::Outcome(stop),
                    cleanup: clean_out,
                }
            }
            bad => {
                if let Some(d) = failure_detail(&bad) {
                    let id = "failure-0001".to_owned();
                    first = Some(FailureObjectV1 {
                        id: id.clone(),
                        phase: FailurePhaseV1::PerfOpen,
                        source: FailureSourceV1::Perf(event),
                        detail: d,
                    });
                }
                let id = first.as_ref().unwrap().id.clone();
                PerfEventV1 {
                    event,
                    scope: ScopeV1::MeasuredThread,
                    unit: UnitV1::PerfCounter,
                    open: PreflightOutcomeV1::Outcome(match bad {
                        Outcome::Unavailable(v) => Outcome::Unavailable(v),
                        Outcome::Permission(v) => Outcome::Permission(v),
                        Outcome::Overflow(v) => Outcome::Overflow(v),
                        Outcome::Error(v) => Outcome::Error(v),
                        Outcome::Success(_) => unreachable!(),
                    }),
                    stop_read: skipped(&id),
                    cleanup: skipped(&id),
                }
            }
        }
    });
    let first_cleanup_id = if first.is_some() { 2 } else { 1 };
    for (offset, failure) in cleanup_failures.iter_mut().enumerate() {
        failure.id = format!("failure-{:04}", first_cleanup_id + offset);
    }
    let classification = classify(first.as_ref(), &cleanup_failures);
    TargetPreflightArtifactV2 {
        schema: TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2,
        repository_revision: r.repository_revision.into(),
        build_identity: r.build_identity.into(),
        platform: TargetPlatformV2 {
            expected: r.expected_platform,
            disposition: PlatformDispositionV2::ProspectiveFedora44Linux,
        },
        architecture: TargetArchitectureV2 {
            expected: r.expected_architecture,
            observed: ArchitectureObservationV2::X86_64,
        },
        measured_file: MeasuredFileV1 {
            identity: r.measured_file_identity.into(),
            open: PreflightOutcomeV1::Outcome(Outcome::Success(())),
            regular_file: PreflightOutcomeV1::Outcome(Outcome::Success(true)),
            length: base_source(
                SourceIdentityV1::FileLength,
                ScopeV1::MeasuredFile,
                UnitV1::Bytes,
                fl,
            ),
        },
        sources: SourcesV1 {
            clock_resolution_realtime: base_source(
                SourceIdentityV1::ClockResolutionRealtime,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
                rr,
            ),
            clock_resolution_monotonic_raw: base_source(
                SourceIdentityV1::ClockResolutionMonotonicRaw,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
                rm,
            ),
            realtime: base_source(
                SourceIdentityV1::Realtime,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
                realtime,
            ),
            monotonic_raw: base_source(
                SourceIdentityV1::MonotonicRaw,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
                mono,
            ),
            process_rusage: base_source(
                SourceIdentityV1::ProcessRusage,
                ScopeV1::Process,
                UnitV1::ResourceUsage,
                pr,
            ),
            thread_rusage: base_source(
                SourceIdentityV1::ThreadRusage,
                ScopeV1::MeasuredThread,
                UnitV1::ResourceUsage,
                tr,
            ),
            statm: base_source(
                SourceIdentityV1::Statm,
                ScopeV1::Process,
                UnitV1::StatmPages,
                statm,
            ),
            status: base_source(
                SourceIdentityV1::Status,
                ScopeV1::Process,
                UnitV1::Bytes,
                status,
            ),
            process_io: base_source(
                SourceIdentityV1::ProcessIo,
                ScopeV1::Process,
                UnitV1::ProcessIo,
                pio,
            ),
        },
        perf_events,
        lifecycle: TargetPreflightLifecycleV2 {
            phases: vec![
                LifecyclePhaseV1::RequestValidated,
                LifecyclePhaseV1::FileOpened,
                LifecyclePhaseV1::SourcesChecked,
                LifecyclePhaseV1::PerfChecked,
            ],
            measured_file_ownership_release: MeasuredFileOwnershipReleaseV2::NotAcquired,
        },
        first_causal_failure: first,
        cleanup_failures,
        tracefs: TracefsV1 {
            state: TracefsStateV1::NotCollected,
            reason: TRACEFS_REASON_V1,
        },
        classification,
        reasons: Vec::new(),
    }
}
fn classify(
    first: Option<&FailureObjectV1>,
    cleanup: &[FailureObjectV1],
) -> ArtifactClassificationV1 {
    if !cleanup.is_empty()
        || first.is_some_and(|f| {
            matches!(
                f.detail,
                FailureDetailV1::Overflow(_)
                    | FailureDetailV1::Error(_)
                    | FailureDetailV1::InvalidState(_)
            )
        })
    {
        ArtifactClassificationV1::Invalid
    } else if first.is_some() {
        ArtifactClassificationV1::Blocked
    } else {
        ArtifactClassificationV1::PreflightSubsetReady
    }
}
fn empty_artifact(
    r: &TargetPreflightRequest<'_>,
    first: FailureObjectV1,
    open: PreflightOutcomeV1<()>,
    regular: PreflightOutcomeV1<bool>,
) -> TargetPreflightArtifactV2 {
    let id = first.id.clone();
    macro_rules! s {
        ($identity:expr,$scope:expr,$unit:expr $(,)?) => {
            base_source($identity, $scope, $unit, skipped(&id))
        };
    }
    TargetPreflightArtifactV2 {
        schema: TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2,
        repository_revision: r.repository_revision.into(),
        build_identity: r.build_identity.into(),
        platform: TargetPlatformV2 {
            expected: r.expected_platform,
            disposition: PlatformDispositionV2::ProspectiveFedora44Linux,
        },
        architecture: TargetArchitectureV2 {
            expected: r.expected_architecture,
            observed: ArchitectureObservationV2::X86_64,
        },
        measured_file: MeasuredFileV1 {
            identity: r.measured_file_identity.into(),
            open,
            regular_file: regular,
            length: s!(
                SourceIdentityV1::FileLength,
                ScopeV1::MeasuredFile,
                UnitV1::Bytes,
            ),
        },
        sources: SourcesV1 {
            clock_resolution_realtime: s!(
                SourceIdentityV1::ClockResolutionRealtime,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
            ),
            clock_resolution_monotonic_raw: s!(
                SourceIdentityV1::ClockResolutionMonotonicRaw,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
            ),
            realtime: s!(
                SourceIdentityV1::Realtime,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
            ),
            monotonic_raw: s!(
                SourceIdentityV1::MonotonicRaw,
                ScopeV1::Observation,
                UnitV1::Nanoseconds,
            ),
            process_rusage: s!(
                SourceIdentityV1::ProcessRusage,
                ScopeV1::Process,
                UnitV1::ResourceUsage,
            ),
            thread_rusage: s!(
                SourceIdentityV1::ThreadRusage,
                ScopeV1::MeasuredThread,
                UnitV1::ResourceUsage,
            ),
            statm: s!(
                SourceIdentityV1::Statm,
                ScopeV1::Process,
                UnitV1::StatmPages,
            ),
            status: s!(SourceIdentityV1::Status, ScopeV1::Process, UnitV1::Bytes),
            process_io: s!(
                SourceIdentityV1::ProcessIo,
                ScopeV1::Process,
                UnitV1::ProcessIo,
            ),
        },
        perf_events: [
            PerfEvent::CpuCycles,
            PerfEvent::Instructions,
            PerfEvent::PageFaults,
            PerfEvent::ContextSwitches,
        ]
        .map(|event| PerfEventV1 {
            event,
            scope: ScopeV1::MeasuredThread,
            unit: UnitV1::PerfCounter,
            open: skipped(&id),
            stop_read: skipped(&id),
            cleanup: skipped(&id),
        }),
        lifecycle: TargetPreflightLifecycleV2 {
            phases: vec![LifecyclePhaseV1::RequestValidated],
            measured_file_ownership_release: MeasuredFileOwnershipReleaseV2::NotAcquired,
        },
        first_causal_failure: Some(first),
        cleanup_failures: vec![],
        tracefs: TracefsV1 {
            state: TracefsStateV1::NotCollected,
            reason: TRACEFS_REASON_V1,
        },
        classification: ArtifactClassificationV1::Invalid,
        reasons: vec![],
    }
}
fn build_open_failure(r: &TargetPreflightRequest<'_>, e: io::Error) -> TargetPreflightArtifactV2 {
    let d = FailureDetailV1::Error(ErrorReason::Io(e.kind()));
    empty_artifact(
        r,
        FailureObjectV1 {
            id: "failure-0001".into(),
            phase: FailurePhaseV1::MeasuredFileOpen,
            source: FailureSourceV1::MeasuredFile,
            detail: d.clone(),
        },
        PreflightOutcomeV1::Outcome(match d {
            FailureDetailV1::Error(x) => Outcome::Error(x),
            _ => unreachable!(),
        }),
        skipped("failure-0001"),
    )
}
fn mark_drop_completed(mut artifact: TargetPreflightArtifactV2) -> TargetPreflightArtifactV2 {
    artifact
        .lifecycle
        .phases
        .push(LifecyclePhaseV1::OwnershipReleased);
    artifact.lifecycle.measured_file_ownership_release =
        MeasuredFileOwnershipReleaseV2::DropCompleted;
    artifact
}

fn mark_file_opened(mut artifact: TargetPreflightArtifactV2) -> TargetPreflightArtifactV2 {
    artifact.lifecycle.phases.push(LifecyclePhaseV1::FileOpened);
    artifact
}

fn build_metadata_failure(
    r: &TargetPreflightRequest<'_>,
    e: io::Error,
) -> TargetPreflightArtifactV2 {
    let d = FailureDetailV1::Error(ErrorReason::Io(e.kind()));
    mark_file_opened(empty_artifact(
        r,
        FailureObjectV1 {
            id: "failure-0001".into(),
            phase: FailurePhaseV1::MeasuredFileRegularFile,
            source: FailureSourceV1::MeasuredFile,
            detail: d.clone(),
        },
        PreflightOutcomeV1::Outcome(Outcome::Success(())),
        PreflightOutcomeV1::Outcome(match d {
            FailureDetailV1::Error(x) => Outcome::Error(x),
            _ => unreachable!(),
        }),
    ))
}

fn build_nonregular(r: &TargetPreflightRequest<'_>) -> TargetPreflightArtifactV2 {
    mark_file_opened(empty_artifact(
        r,
        FailureObjectV1 {
            id: "failure-0001".into(),
            phase: FailurePhaseV1::MeasuredFileRegularFile,
            source: FailureSourceV1::MeasuredFile,
            detail: FailureDetailV1::InvalidState(InvalidStateReasonV1::NotRegularFile),
        },
        PreflightOutcomeV1::Outcome(Outcome::Success(())),
        PreflightOutcomeV1::Outcome(Outcome::Success(false)),
    ))
}
fn finish_artifact(
    artifact: TargetPreflightArtifactV2,
    sink: &mut dyn TargetPreflightRetention,
) -> TargetPreflightExecutionV2 {
    let bytes = match serialize(&artifact) {
        Ok(x) => x,
        Err(()) => {
            return TargetPreflightExecutionV2 {
                artifact: Some(artifact),
                serialized_bytes: None,
                retention: RetentionOutcomeV1::NotAttempted {
                    reason: NotAttemptedReasonV1::SerializationFailure,
                },
                disposition: TargetPreflightCallDispositionV1::SerializationFailed,
            };
        }
    };
    let len = bytes.len() as u64;
    let result = match sink.write_all(&bytes) {
        Err(e) => Err((RetentionOperationV1::WriteAll, e)),
        Ok(()) => sink.flush().map_err(|e| (RetentionOperationV1::Flush, e)),
    };
    match result {
        Ok(()) => TargetPreflightExecutionV2 {
            artifact: Some(artifact),
            serialized_bytes: Some(bytes),
            retention: RetentionOutcomeV1::Success {
                serialized_byte_length: len,
            },
            disposition: TargetPreflightCallDispositionV1::Completed,
        },
        Err((operation, e)) => TargetPreflightExecutionV2 {
            artifact: Some(artifact),
            serialized_bytes: Some(bytes),
            retention: RetentionOutcomeV1::IoFailure {
                operation,
                error: RetentionIoErrorV1 {
                    kind: map_io(e.kind()),
                    raw_os_error: e.raw_os_error(),
                },
            },
            disposition: TargetPreflightCallDispositionV1::RetentionFailed,
        },
    }
}

fn map_io(k: io::ErrorKind) -> IoErrorKindV1 {
    use io::ErrorKind as E;
    match k {
        E::NotFound => IoErrorKindV1::NotFound,
        E::PermissionDenied => IoErrorKindV1::PermissionDenied,
        E::ConnectionRefused => IoErrorKindV1::ConnectionRefused,
        E::ConnectionReset => IoErrorKindV1::ConnectionReset,
        E::HostUnreachable => IoErrorKindV1::HostUnreachable,
        E::NetworkUnreachable => IoErrorKindV1::NetworkUnreachable,
        E::ConnectionAborted => IoErrorKindV1::ConnectionAborted,
        E::NotConnected => IoErrorKindV1::NotConnected,
        E::AddrInUse => IoErrorKindV1::AddrInUse,
        E::AddrNotAvailable => IoErrorKindV1::AddrNotAvailable,
        E::NetworkDown => IoErrorKindV1::NetworkDown,
        E::BrokenPipe => IoErrorKindV1::BrokenPipe,
        E::AlreadyExists => IoErrorKindV1::AlreadyExists,
        E::WouldBlock => IoErrorKindV1::WouldBlock,
        E::NotADirectory => IoErrorKindV1::NotADirectory,
        E::IsADirectory => IoErrorKindV1::IsADirectory,
        E::DirectoryNotEmpty => IoErrorKindV1::DirectoryNotEmpty,
        E::ReadOnlyFilesystem => IoErrorKindV1::ReadOnlyFilesystem,
        E::StaleNetworkFileHandle => IoErrorKindV1::StaleNetworkFileHandle,
        E::InvalidInput => IoErrorKindV1::InvalidInput,
        E::InvalidData => IoErrorKindV1::InvalidData,
        E::TimedOut => IoErrorKindV1::TimedOut,
        E::WriteZero => IoErrorKindV1::WriteZero,
        E::StorageFull => IoErrorKindV1::StorageFull,
        E::NotSeekable => IoErrorKindV1::NotSeekable,
        E::QuotaExceeded => IoErrorKindV1::QuotaExceeded,
        E::FileTooLarge => IoErrorKindV1::FileTooLarge,
        E::ResourceBusy => IoErrorKindV1::ResourceBusy,
        E::ExecutableFileBusy => IoErrorKindV1::ExecutableFileBusy,
        E::Deadlock => IoErrorKindV1::Deadlock,
        E::CrossesDevices => IoErrorKindV1::CrossesDevices,
        E::TooManyLinks => IoErrorKindV1::TooManyLinks,
        E::InvalidFilename => IoErrorKindV1::InvalidFilename,
        E::ArgumentListTooLong => IoErrorKindV1::ArgumentListTooLong,
        E::Interrupted => IoErrorKindV1::Interrupted,
        E::Unsupported => IoErrorKindV1::Unsupported,
        E::UnexpectedEof => IoErrorKindV1::UnexpectedEof,
        E::OutOfMemory => IoErrorKindV1::OutOfMemory,
        // `FilesystemLoop` and `InProgress` remain unstable on pinned Rust 1.89.0;
        // the non-exhaustive fallback is therefore the only representable mapping here.
        _ => IoErrorKindV1::Other,
    }
}

trait JsonValue {
    fn json(&self, o: &mut String) -> Result<(), ()>;
}
fn q(o: &mut String, s: &str) -> Result<(), ()> {
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            '\u{08}' => o.push_str("\\b"),
            '\u{0c}' => o.push_str("\\f"),
            c if c.is_ascii() && !c.is_ascii_control() => o.push(c),
            c if c.is_ascii() => {
                use std::fmt::Write;
                write!(o, "\\u{:04x}", c as u32).unwrap()
            }
            _ => return Err(()),
        }
    }
    o.push('"');
    Ok(())
}
macro_rules! num {($($t:ty),*)=>{$(impl JsonValue for $t{fn json(&self,o:&mut String)->Result<(),()>{use std::fmt::Write;write!(o,"{self}").unwrap();Ok(())}})*};}
num!(i32, i64, i128, isize, u64);
impl JsonValue for bool {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        o.push_str(if *self { "true" } else { "false" });
        Ok(())
    }
}
impl JsonValue for () {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        o.push_str("null");
        Ok(())
    }
}
fn event(e: PerfEvent) -> &'static str {
    match e {
        PerfEvent::CpuCycles => "cpu_cycles",
        PerfEvent::Instructions => "instructions",
        PerfEvent::PageFaults => "page_faults",
        PerfEvent::ContextSwitches => "context_switches",
    }
}
fn source(s: SourceIdentityV1) -> &'static str {
    match s {
        SourceIdentityV1::ClockResolutionRealtime => "clock_resolution_realtime",
        SourceIdentityV1::ClockResolutionMonotonicRaw => "clock_resolution_monotonic_raw",
        SourceIdentityV1::Realtime => "realtime",
        SourceIdentityV1::MonotonicRaw => "monotonic_raw",
        SourceIdentityV1::ProcessRusage => "process_rusage",
        SourceIdentityV1::ThreadRusage => "thread_rusage",
        SourceIdentityV1::Statm => "statm",
        SourceIdentityV1::Status => "status",
        SourceIdentityV1::ProcessIo => "process_io",
        SourceIdentityV1::FileLength => "file_length",
    }
}
impl<T: JsonValue> JsonValue for PreflightOutcomeV1<T> {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        match self {
            Self::Outcome(v) => v.json(o)?,
            Self::NotAttempted { failure_id } => {
                o.push_str("{\"not_attempted\":{\"failure_id\":");
                q(o, failure_id)?;
                o.push_str("}}");
            }
        }
        Ok(())
    }
}
impl<T: JsonValue> JsonValue for Outcome<T> {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        match self {
            Outcome::Success(v) => {
                o.push_str("{\"success\":");
                v.json(o)?
            }
            Outcome::Unavailable(v) => {
                o.push_str("{\"unavailable\":");
                unavailable(o, *v)
            }
            Outcome::Permission(v) => {
                o.push_str("{\"permission\":{\"errno\":");
                v.json(o)?;
                o.push('}')
            }
            Outcome::Overflow(v) => {
                o.push_str("{\"overflow\":");
                overflow(o, *v)
            }
            Outcome::Error(v) => {
                o.push_str("{\"error\":");
                error(o, v)?
            }
        }
        o.push('}');
        Ok(())
    }
}
fn unavailable(o: &mut String, v: UnavailableReason) {
    match v {
        UnavailableReason::Interface(e) => {
            o.push_str("{\"kind\":\"interface\",\"errno\":");
            e.json(o).unwrap();
            o.push('}')
        }
        UnavailableReason::MissingStatxSize => o.push_str("{\"kind\":\"missing_statx_size\"}"),
        UnavailableReason::NotFound => o.push_str("{\"kind\":\"not_found\"}"),
        UnavailableReason::Unsupported => o.push_str("{\"kind\":\"unsupported\"}"),
        UnavailableReason::StatxOnlyAfterFstat => {
            o.push_str("{\"kind\":\"statx_only_after_fstat\"}")
        }
    }
}
fn overflow(o: &mut String, v: OverflowReason) {
    match v {
        OverflowReason::Arithmetic => o.push_str("{\"kind\":\"arithmetic\"}"),
        OverflowReason::FileSize => o.push_str("{\"kind\":\"file_size\"}"),
        OverflowReason::NumericField => o.push_str("{\"kind\":\"numeric_field\"}"),
        OverflowReason::PerfScaling => o.push_str("{\"kind\":\"perf_scaling\"}"),
        OverflowReason::PerfErrno(e) => {
            o.push_str("{\"kind\":\"perf_errno\",\"errno\":");
            e.json(o).unwrap();
            o.push('}')
        }
    }
}
fn io_name(k: io::ErrorKind) -> &'static str {
    match map_io(k) {
        IoErrorKindV1::NotFound => "not_found",
        IoErrorKindV1::PermissionDenied => "permission_denied",
        IoErrorKindV1::ConnectionRefused => "connection_refused",
        IoErrorKindV1::ConnectionReset => "connection_reset",
        IoErrorKindV1::HostUnreachable => "host_unreachable",
        IoErrorKindV1::NetworkUnreachable => "network_unreachable",
        IoErrorKindV1::ConnectionAborted => "connection_aborted",
        IoErrorKindV1::NotConnected => "not_connected",
        IoErrorKindV1::AddrInUse => "addr_in_use",
        IoErrorKindV1::AddrNotAvailable => "addr_not_available",
        IoErrorKindV1::NetworkDown => "network_down",
        IoErrorKindV1::BrokenPipe => "broken_pipe",
        IoErrorKindV1::AlreadyExists => "already_exists",
        IoErrorKindV1::WouldBlock => "would_block",
        IoErrorKindV1::NotADirectory => "not_a_directory",
        IoErrorKindV1::IsADirectory => "is_a_directory",
        IoErrorKindV1::DirectoryNotEmpty => "directory_not_empty",
        IoErrorKindV1::ReadOnlyFilesystem => "read_only_filesystem",
        IoErrorKindV1::FilesystemLoop => "filesystem_loop",
        IoErrorKindV1::StaleNetworkFileHandle => "stale_network_file_handle",
        IoErrorKindV1::InvalidInput => "invalid_input",
        IoErrorKindV1::InvalidData => "invalid_data",
        IoErrorKindV1::TimedOut => "timed_out",
        IoErrorKindV1::WriteZero => "write_zero",
        IoErrorKindV1::StorageFull => "storage_full",
        IoErrorKindV1::NotSeekable => "not_seekable",
        IoErrorKindV1::QuotaExceeded => "quota_exceeded",
        IoErrorKindV1::FileTooLarge => "file_too_large",
        IoErrorKindV1::ResourceBusy => "resource_busy",
        IoErrorKindV1::ExecutableFileBusy => "executable_file_busy",
        IoErrorKindV1::Deadlock => "deadlock",
        IoErrorKindV1::CrossesDevices => "crosses_devices",
        IoErrorKindV1::TooManyLinks => "too_many_links",
        IoErrorKindV1::InvalidFilename => "invalid_filename",
        IoErrorKindV1::ArgumentListTooLong => "argument_list_too_long",
        IoErrorKindV1::Interrupted => "interrupted",
        IoErrorKindV1::Unsupported => "unsupported",
        IoErrorKindV1::UnexpectedEof => "unexpected_eof",
        IoErrorKindV1::OutOfMemory => "out_of_memory",
        IoErrorKindV1::InProgress => "in_progress",
        IoErrorKindV1::Other => "other",
    }
}
fn parse_name(p: ParseReason) -> &'static str {
    match p {
        ParseReason::NonAscii => "non_ascii",
        ParseReason::LineCount => "line_count",
        ParseReason::TokenCount => "token_count",
        ParseReason::MalformedLine => "malformed_line",
        ParseReason::MissingField => "missing_field",
        ParseReason::DuplicateField => "duplicate_field",
        ParseReason::SignedValue => "signed_value",
        ParseReason::InvalidNumber => "invalid_number",
        ParseReason::InvalidUnit => "invalid_unit",
        ParseReason::TrailingToken => "trailing_token",
    }
}
fn error(o: &mut String, v: &ErrorReason) -> Result<(), ()> {
    let simple = match v {
        ErrorReason::InvalidFraction => Some("invalid_fraction"),
        ErrorReason::NegativeCounter => Some("negative_counter"),
        ErrorReason::NegativeFileSize => Some("negative_file_size"),
        ErrorReason::InvalidUtf8 => Some("invalid_utf8"),
        ErrorReason::PerfInvalidTime => Some("perf_invalid_time"),
        ErrorReason::PerfDecrease => Some("perf_decrease"),
        ErrorReason::PerfLifecycle => Some("perf_lifecycle"),
        ErrorReason::MissingFileCapability => Some("missing_file_capability"),
        _ => None,
    };
    if let Some(k) = simple {
        o.push_str("{\"kind\":");
        q(o, k)?;
        o.push('}');
        return Ok(());
    }
    match v {
        ErrorReason::Errno(x) => detail_i(o, "errno", "errno", *x),
        ErrorReason::Io(k) => {
            o.push_str("{\"kind\":\"io\",\"error_kind\":");
            q(o, io_name(*k))?;
            o.push('}')
        }
        ErrorReason::Parse(p) => {
            o.push_str("{\"kind\":\"parse\",\"reason\":");
            q(o, parse_name(*p))?;
            o.push('}')
        }
        ErrorReason::PerfShortRead(x) => detail_i(o, "perf_short_read", "actual", *x),
        ErrorReason::PerfCleanup(x) => detail_i(o, "perf_cleanup", "errno", *x),
        ErrorReason::PerfUnexpectedReturn(x) => detail_i(o, "perf_unexpected_return", "actual", *x),
        ErrorReason::PerfCleanupUnexpected(x) => {
            detail_i(o, "perf_cleanup_unexpected", "actual", *x)
        }
        ErrorReason::PerfEventMismatch { expected, actual } => {
            o.push_str("{\"kind\":\"perf_event_mismatch\",\"expected\":");
            q(o, event(*expected))?;
            o.push_str(",\"actual\":");
            q(o, event(*actual))?;
            o.push('}')
        }
        _ => unreachable!(),
    }
    Ok(())
}
fn detail_i<T: JsonValue>(o: &mut String, k: &str, n: &str, v: T) {
    o.push_str("{\"kind\":\"");
    o.push_str(k);
    o.push_str("\",\"");
    o.push_str(n);
    o.push_str("\":");
    v.json(o).unwrap();
    o.push('}')
}
impl JsonValue for ResourceUsage {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        use std::fmt::Write;
        write!(o,"{{\"user_nanoseconds\":{},\"system_nanoseconds\":{},\"maximum_resident_bytes\":{},\"minor_faults\":{},\"major_faults\":{},\"input_blocks\":{},\"output_blocks\":{},\"voluntary_context_switches\":{},\"involuntary_context_switches\":{}}}",self.user_nanoseconds,self.system_nanoseconds,self.maximum_resident_bytes,self.minor_faults,self.major_faults,self.input_blocks,self.output_blocks,self.voluntary_context_switches,self.involuntary_context_switches).unwrap();
        Ok(())
    }
}
impl JsonValue for Statm {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        use std::fmt::Write;
        write!(o,"{{\"size\":{},\"resident\":{},\"shared\":{},\"text\":{},\"lib\":{},\"data\":{},\"dt\":{}}}",self.size,self.resident,self.shared,self.text,self.lib,self.data,self.dt).unwrap();
        Ok(())
    }
}
impl JsonValue for StatusMemory {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        use std::fmt::Write;
        write!(
            o,
            "{{\"resident_bytes\":{},\"high_water_bytes\":{}}}",
            self.resident_bytes, self.high_water_bytes
        )
        .unwrap();
        Ok(())
    }
}
impl JsonValue for ProcessIo {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        use std::fmt::Write;
        write!(o,"{{\"rchar\":{},\"wchar\":{},\"syscr\":{},\"syscw\":{},\"read_bytes\":{},\"write_bytes\":{},\"cancelled_write_bytes\":{}}}",self.rchar,self.wchar,self.syscr,self.syscw,self.read_bytes,self.write_bytes,self.cancelled_write_bytes).unwrap();
        Ok(())
    }
}
impl JsonValue for FileLength {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        use std::fmt::Write;
        write!(
            o,
            "{{\"bytes\":{},\"source\":\"{}\",\"statx_only_fields\":",
            self.bytes,
            match self.source {
                FileLengthSource::Statx => "statx",
                FileLengthSource::FstatFallback => "fstat_fallback",
            }
        )
        .unwrap();
        match self.statx_only_fields {
            Ok(()) => o.push_str("{\"success\":null}"),
            Err(v) => {
                o.push_str("{\"unavailable\":");
                unavailable(o, v);
                o.push('}')
            }
        }
        o.push('}');
        Ok(())
    }
}
impl JsonValue for PerfCounter {
    fn json(&self, o: &mut String) -> Result<(), ()> {
        use std::fmt::Write;
        write!(o,"{{\"event\":\"{}\",\"raw_count\":{},\"time_enabled_ns\":{},\"time_running_ns\":{},\"multiplexed\":{},\"scaled_count\":",event(self.event),self.raw_count,self.time_enabled_ns,self.time_running_ns,self.multiplexed).unwrap();
        self.scaled_count.json(o)?;
        o.push('}');
        Ok(())
    }
}
fn unit(o: &mut String, u: UnitV1) {
    match u{UnitV1::Nanoseconds=>o.push_str("\"nanoseconds\""),UnitV1::Bytes=>o.push_str("\"bytes\""),UnitV1::StatmPages=>o.push_str("\"statm_pages\""),UnitV1::ResourceUsage=>o.push_str("{\"resource_usage\":{\"user\":\"nanoseconds\",\"system\":\"nanoseconds\",\"maximum_resident\":\"bytes\",\"minor_faults\":\"events\",\"major_faults\":\"events\",\"input_blocks\":\"operations\",\"output_blocks\":\"operations\",\"voluntary_context_switches\":\"events\",\"involuntary_context_switches\":\"events\"}}"),UnitV1::ProcessIo=>o.push_str("{\"process_io\":{\"rchar\":\"bytes\",\"wchar\":\"bytes\",\"syscr\":\"operations\",\"syscw\":\"operations\",\"read\":\"bytes\",\"write\":\"bytes\",\"cancelled_write\":\"bytes\"}}"),UnitV1::PerfCounter=>o.push_str("{\"perf_counter\":{\"raw_count\":\"events\",\"time_enabled\":\"nanoseconds\",\"time_running\":\"nanoseconds\",\"scaled_count\":\"events\"}}")}
}
fn scope(s: ScopeV1) -> &'static str {
    match s {
        ScopeV1::Observation => "observation",
        ScopeV1::Process => "process",
        ScopeV1::MeasuredThread => "measured_thread",
        ScopeV1::MeasuredFile => "measured_file",
    }
}
fn source_json<T: JsonValue>(o: &mut String, s: &SourceV1<T>) -> Result<(), ()> {
    o.push_str("{\"identity\":");
    q(o, source(s.identity))?;
    o.push_str(",\"scope\":");
    q(o, scope(s.scope))?;
    o.push_str(",\"unit\":");
    unit(o, s.unit);
    o.push_str(",\"outcome\":");
    s.outcome.json(o)?;
    o.push('}');
    Ok(())
}
fn perf_json(o: &mut String, p: &PerfEventV1) -> Result<(), ()> {
    o.push_str("{\"event\":");
    q(o, event(p.event))?;
    o.push_str(",\"scope\":");
    q(o, scope(p.scope))?;
    o.push_str(",\"unit\":");
    unit(o, p.unit);
    o.push_str(",\"open\":");
    p.open.json(o)?;
    o.push_str(",\"stop_read\":");
    p.stop_read.json(o)?;
    o.push_str(",\"cleanup\":");
    p.cleanup.json(o)?;
    o.push('}');
    Ok(())
}
fn failure_json(o: &mut String, f: &FailureObjectV1) -> Result<(), ()> {
    o.push_str("{\"id\":");
    q(o, &f.id)?;
    o.push_str(",\"phase\":");
    q(
        o,
        match f.phase {
            FailurePhaseV1::PlatformValidation => "platform_validation",
            FailurePhaseV1::ArchitectureValidation => "architecture_validation",
            FailurePhaseV1::MeasuredFileOpen => "measured_file_open",
            FailurePhaseV1::MeasuredFileRegularFile => "measured_file_regular_file",
            FailurePhaseV1::ClockResolutionRealtime => "clock_resolution_realtime",
            FailurePhaseV1::ClockResolutionMonotonicRaw => "clock_resolution_monotonic_raw",
            FailurePhaseV1::SourceCapture => "source_capture",
            FailurePhaseV1::PerfOpen => "perf_open",
            FailurePhaseV1::PerfStopRead => "perf_stop_read",
            FailurePhaseV1::PerfCleanup => "perf_cleanup",
            FailurePhaseV1::OwnershipRelease => "ownership_release",
        },
    )?;
    o.push_str(",\"source\":");
    q(
        o,
        match f.source {
            FailureSourceV1::Platform => "platform",
            FailureSourceV1::Architecture => "architecture",
            FailureSourceV1::MeasuredFile => "measured_file",
            FailureSourceV1::Source(s) => source(s),
            FailureSourceV1::Perf(e) => event(e),
            FailureSourceV1::Lifecycle => "lifecycle",
        },
    )?;
    o.push_str(",\"class\":");
    let class = match f.detail {
        FailureDetailV1::Unavailable(_) => "unavailable",
        FailureDetailV1::Permission(_) => "permission",
        FailureDetailV1::Overflow(_) => "overflow",
        FailureDetailV1::Error(_) => "error",
        FailureDetailV1::InvalidState(_) => "invalid_state",
    };
    q(o, class)?;
    o.push_str(",\"detail\":");
    match &f.detail {
        FailureDetailV1::Unavailable(v) => unavailable(o, *v),
        FailureDetailV1::Permission(e) => {
            o.push_str("{\"errno\":");
            e.json(o)?;
            o.push('}')
        }
        FailureDetailV1::Overflow(v) => overflow(o, *v),
        FailureDetailV1::Error(v) => error(o, v)?,
        FailureDetailV1::InvalidState(v) => {
            o.push_str("{\"kind\":");
            q(
                o,
                match v {
                    InvalidStateReasonV1::PlatformMismatch => "platform_mismatch",
                    InvalidStateReasonV1::ArchitectureMismatch => "architecture_mismatch",
                    InvalidStateReasonV1::NotRegularFile => "not_regular_file",
                    InvalidStateReasonV1::LifecycleViolation => "lifecycle_violation",
                    InvalidStateReasonV1::OwnershipReleaseIncomplete => {
                        "ownership_release_incomplete"
                    }
                },
            )?;
            o.push('}')
        }
    }
    o.push('}');
    Ok(())
}
fn valid_text_identity(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_graphic() && b != b'/' && b != b'\\')
}

fn valid_file_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.is_ascii()
        && value
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        && !value.contains("..")
        && !value.contains("://")
}

fn outcome_detail<T>(outcome: &PreflightOutcomeV1<T>) -> Option<FailureDetailV1> {
    match outcome {
        PreflightOutcomeV1::Outcome(value) => failure_detail(value),
        PreflightOutcomeV1::NotAttempted { .. } => None,
    }
}

fn check_step<T>(
    outcome: &PreflightOutcomeV1<T>,
    phase: FailurePhaseV1,
    source: FailureSourceV1,
    causal: Option<&FailureObjectV1>,
    seen: &mut bool,
) -> Result<(), ()> {
    match outcome {
        PreflightOutcomeV1::NotAttempted { failure_id } => {
            let failure = causal.filter(|_| *seen).ok_or(())?;
            if failure_id != &failure.id {
                return Err(());
            }
        }
        PreflightOutcomeV1::Outcome(value) => {
            if *seen {
                return Err(());
            }
            if let Some(detail) = failure_detail(value) {
                let failure = causal.ok_or(())?;
                if failure.phase != phase || failure.source != source || failure.detail != detail {
                    return Err(());
                }
                *seen = true;
            }
        }
    }
    Ok(())
}

fn check_skipped<T>(outcome: &PreflightOutcomeV1<T>, id: &str) -> bool {
    matches!(outcome, PreflightOutcomeV1::NotAttempted { failure_id } if failure_id == id)
}

fn validate_artifact(a: &TargetPreflightArtifactV2) -> Result<(), ()> {
    if a.schema != TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2
        || a.repository_revision.len() != 40
        || !a
            .repository_revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !valid_text_identity(&a.build_identity, 128)
        || !valid_file_identity(&a.measured_file.identity)
        || a.platform.expected != TargetPlatformV1::Fedora44Linux
        || a.platform.disposition != PlatformDispositionV2::ProspectiveFedora44Linux
        || a.architecture.expected != TargetArchitectureV1::X86_64
        || a.architecture.observed != ArchitectureObservationV2::X86_64
        || a.tracefs.state != TracefsStateV1::NotCollected
        || a.tracefs.reason != TRACEFS_REASON_V1
        || !a.reasons.is_empty()
    {
        return Err(());
    }

    macro_rules! metadata {
        ($field:expr,$identity:expr,$scope:expr,$unit:expr) => {
            if $field.identity != $identity || $field.scope != $scope || $field.unit != $unit {
                return Err(());
            }
        };
    }
    metadata!(
        a.measured_file.length,
        SourceIdentityV1::FileLength,
        ScopeV1::MeasuredFile,
        UnitV1::Bytes
    );
    metadata!(
        a.sources.clock_resolution_realtime,
        SourceIdentityV1::ClockResolutionRealtime,
        ScopeV1::Observation,
        UnitV1::Nanoseconds
    );
    metadata!(
        a.sources.clock_resolution_monotonic_raw,
        SourceIdentityV1::ClockResolutionMonotonicRaw,
        ScopeV1::Observation,
        UnitV1::Nanoseconds
    );
    metadata!(
        a.sources.realtime,
        SourceIdentityV1::Realtime,
        ScopeV1::Observation,
        UnitV1::Nanoseconds
    );
    metadata!(
        a.sources.monotonic_raw,
        SourceIdentityV1::MonotonicRaw,
        ScopeV1::Observation,
        UnitV1::Nanoseconds
    );
    metadata!(
        a.sources.process_rusage,
        SourceIdentityV1::ProcessRusage,
        ScopeV1::Process,
        UnitV1::ResourceUsage
    );
    metadata!(
        a.sources.thread_rusage,
        SourceIdentityV1::ThreadRusage,
        ScopeV1::MeasuredThread,
        UnitV1::ResourceUsage
    );
    metadata!(
        a.sources.statm,
        SourceIdentityV1::Statm,
        ScopeV1::Process,
        UnitV1::StatmPages
    );
    metadata!(
        a.sources.status,
        SourceIdentityV1::Status,
        ScopeV1::Process,
        UnitV1::Bytes
    );
    metadata!(
        a.sources.process_io,
        SourceIdentityV1::ProcessIo,
        ScopeV1::Process,
        UnitV1::ProcessIo
    );
    let events = [
        PerfEvent::CpuCycles,
        PerfEvent::Instructions,
        PerfEvent::PageFaults,
        PerfEvent::ContextSwitches,
    ];
    for (entry, event) in a.perf_events.iter().zip(events) {
        if entry.event != event
            || entry.scope != ScopeV1::MeasuredThread
            || entry.unit != UnitV1::PerfCounter
        {
            return Err(());
        }
        if let PreflightOutcomeV1::Outcome(Outcome::Success(counter)) = &entry.stop_read
            && counter.event != event
        {
            return Err(());
        }
    }

    let causal = a.first_causal_failure.as_ref();
    if causal.is_some_and(|f| f.id != "failure-0001") {
        return Err(());
    }
    let open_success = matches!(
        a.measured_file.open,
        PreflightOutcomeV1::Outcome(Outcome::Success(()))
    );
    let regular_success = matches!(
        a.measured_file.regular_file,
        PreflightOutcomeV1::Outcome(Outcome::Success(true))
    );
    let acquired_early_failure =
        causal.is_some_and(|f| f.phase == FailurePhaseV1::MeasuredFileRegularFile);
    let expected_phases: &[LifecyclePhaseV1] = if !open_success {
        &[LifecyclePhaseV1::RequestValidated]
    } else if acquired_early_failure {
        &[
            LifecyclePhaseV1::RequestValidated,
            LifecyclePhaseV1::FileOpened,
            LifecyclePhaseV1::OwnershipReleased,
        ]
    } else {
        &[
            LifecyclePhaseV1::RequestValidated,
            LifecyclePhaseV1::FileOpened,
            LifecyclePhaseV1::SourcesChecked,
            LifecyclePhaseV1::PerfChecked,
            LifecyclePhaseV1::OwnershipReleased,
        ]
    };
    let expected_ownership = if open_success {
        MeasuredFileOwnershipReleaseV2::DropCompleted
    } else {
        MeasuredFileOwnershipReleaseV2::NotAcquired
    };
    if a.lifecycle.phases != expected_phases
        || a.lifecycle.measured_file_ownership_release != expected_ownership
    {
        return Err(());
    }

    let all_dependent_skipped = |id: &str| {
        check_skipped(&a.measured_file.length.outcome, id)
            && check_skipped(&a.sources.clock_resolution_realtime.outcome, id)
            && check_skipped(&a.sources.clock_resolution_monotonic_raw.outcome, id)
            && check_skipped(&a.sources.realtime.outcome, id)
            && check_skipped(&a.sources.monotonic_raw.outcome, id)
            && check_skipped(&a.sources.process_rusage.outcome, id)
            && check_skipped(&a.sources.thread_rusage.outcome, id)
            && check_skipped(&a.sources.statm.outcome, id)
            && check_skipped(&a.sources.status.outcome, id)
            && check_skipped(&a.sources.process_io.outcome, id)
            && a.perf_events.iter().all(|p| {
                check_skipped(&p.open, id)
                    && check_skipped(&p.stop_read, id)
                    && check_skipped(&p.cleanup, id)
            })
    };

    if !open_success {
        let failure = causal.ok_or(())?;
        if failure.phase != FailurePhaseV1::MeasuredFileOpen
            || failure.source != FailureSourceV1::MeasuredFile
            || outcome_detail(&a.measured_file.open) != Some(failure.detail.clone())
            || !check_skipped(&a.measured_file.regular_file, &failure.id)
            || !all_dependent_skipped(&failure.id)
        {
            return Err(());
        }
    } else if acquired_early_failure {
        let failure = causal.ok_or(())?;
        let regular_matches = match &a.measured_file.regular_file {
            PreflightOutcomeV1::Outcome(Outcome::Success(false)) => {
                failure.detail
                    == FailureDetailV1::InvalidState(InvalidStateReasonV1::NotRegularFile)
            }
            value => outcome_detail(value) == Some(failure.detail.clone()),
        };
        if failure.source != FailureSourceV1::MeasuredFile
            || !regular_matches
            || !all_dependent_skipped(&failure.id)
        {
            return Err(());
        }
    } else {
        if !regular_success {
            return Err(());
        }
        let mut seen = false;
        macro_rules! step {
            ($out:expr,$phase:expr,$source:expr) => {
                check_step(&$out, $phase, $source, causal, &mut seen)?
            };
        }
        step!(
            a.sources.clock_resolution_realtime.outcome,
            FailurePhaseV1::ClockResolutionRealtime,
            FailureSourceV1::Source(SourceIdentityV1::ClockResolutionRealtime)
        );
        step!(
            a.sources.clock_resolution_monotonic_raw.outcome,
            FailurePhaseV1::ClockResolutionMonotonicRaw,
            FailureSourceV1::Source(SourceIdentityV1::ClockResolutionMonotonicRaw)
        );
        step!(
            a.sources.realtime.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::Realtime)
        );
        step!(
            a.sources.monotonic_raw.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::MonotonicRaw)
        );
        step!(
            a.sources.process_rusage.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::ProcessRusage)
        );
        step!(
            a.sources.thread_rusage.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::ThreadRusage)
        );
        step!(
            a.sources.statm.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::Statm)
        );
        step!(
            a.sources.status.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::Status)
        );
        step!(
            a.sources.process_io.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::ProcessIo)
        );
        step!(
            a.measured_file.length.outcome,
            FailurePhaseV1::SourceCapture,
            FailureSourceV1::Source(SourceIdentityV1::FileLength)
        );
        for p in &a.perf_events {
            step!(
                p.open,
                FailurePhaseV1::PerfOpen,
                FailureSourceV1::Perf(p.event)
            );
            step!(
                p.stop_read,
                FailurePhaseV1::PerfStopRead,
                FailureSourceV1::Perf(p.event)
            );
            // Cleanup is independent: it is never the causal failure and remains attempted after stop-read failure.
            if matches!(p.open, PreflightOutcomeV1::Outcome(Outcome::Success(()))) {
                if matches!(p.cleanup, PreflightOutcomeV1::NotAttempted { .. }) {
                    return Err(());
                }
            } else if !check_skipped(&p.cleanup, &causal.ok_or(())?.id) {
                return Err(());
            }
        }
        if seen != causal.is_some() {
            return Err(());
        }
    }

    let expected_first_cleanup = usize::from(causal.is_some()) + 1;
    let mut cleanup_index = 0;
    for p in &a.perf_events {
        if let Some(detail) = outcome_detail(&p.cleanup) {
            let failure = a.cleanup_failures.get(cleanup_index).ok_or(())?;
            if failure.id != format!("failure-{:04}", expected_first_cleanup + cleanup_index)
                || failure.phase != FailurePhaseV1::PerfCleanup
                || failure.source != FailureSourceV1::Perf(p.event)
                || failure.detail != detail
            {
                return Err(());
            }
            cleanup_index += 1;
        }
    }
    if cleanup_index != a.cleanup_failures.len() {
        return Err(());
    }
    if a.classification != classify(causal, &a.cleanup_failures) {
        return Err(());
    }
    Ok(())
}

fn serialize(a: &TargetPreflightArtifactV2) -> Result<Vec<u8>, ()> {
    validate_artifact(a)?;
    let mut o = String::new();
    o.push_str("{\"schema\":");
    q(&mut o, a.schema)?;
    o.push_str(",\"repository_revision\":");
    q(&mut o, &a.repository_revision)?;
    o.push_str(",\"build_identity\":");
    q(&mut o, &a.build_identity)?;
    o.push_str(",\"platform\":{\"expected\":");
    q(
        &mut o,
        match a.platform.expected {
            TargetPlatformV1::Fedora44Linux => "fedora-44-linux",
        },
    )?;
    o.push_str(",\"disposition\":");
    q(
        &mut o,
        match a.platform.disposition {
            PlatformDispositionV2::ProspectiveFedora44Linux => "prospective_fedora_44_linux",
        },
    )?;
    o.push_str("},\"architecture\":{\"expected\":");
    q(
        &mut o,
        match a.architecture.expected {
            TargetArchitectureV1::X86_64 => "x86_64",
        },
    )?;
    o.push_str(",\"observed\":");
    q(
        &mut o,
        match a.architecture.observed {
            ArchitectureObservationV2::X86_64 => "x86_64",
        },
    )?;
    o.push_str("},\"measured_file\":{\"identity\":");
    q(&mut o, &a.measured_file.identity)?;
    o.push_str(",\"open\":");
    a.measured_file.open.json(&mut o)?;
    o.push_str(",\"regular_file\":");
    a.measured_file.regular_file.json(&mut o)?;
    o.push_str(",\"length\":");
    source_json(&mut o, &a.measured_file.length)?;
    o.push_str("},\"sources\":[");
    source_json(&mut o, &a.sources.clock_resolution_realtime)?;
    o.push(',');
    source_json(&mut o, &a.sources.clock_resolution_monotonic_raw)?;
    o.push(',');
    source_json(&mut o, &a.sources.realtime)?;
    o.push(',');
    source_json(&mut o, &a.sources.monotonic_raw)?;
    o.push(',');
    source_json(&mut o, &a.sources.process_rusage)?;
    o.push(',');
    source_json(&mut o, &a.sources.thread_rusage)?;
    o.push(',');
    source_json(&mut o, &a.sources.statm)?;
    o.push(',');
    source_json(&mut o, &a.sources.status)?;
    o.push(',');
    source_json(&mut o, &a.sources.process_io)?;
    o.push_str("],\"perf_events\":[");
    for (i, p) in a.perf_events.iter().enumerate() {
        if i > 0 {
            o.push(',')
        }
        perf_json(&mut o, p)?
    }
    o.push_str("],\"lifecycle\":{\"phases\":[");
    for (i, p) in a.lifecycle.phases.iter().enumerate() {
        if i > 0 {
            o.push(',')
        }
        q(
            &mut o,
            match p {
                LifecyclePhaseV1::RequestValidated => "request_validated",
                LifecyclePhaseV1::FileOpened => "file_opened",
                LifecyclePhaseV1::SourcesChecked => "sources_checked",
                LifecyclePhaseV1::PerfChecked => "perf_checked",
                LifecyclePhaseV1::OwnershipReleased => "ownership_released",
            },
        )?
    }
    o.push_str("],\"measured_file_ownership_release\":");
    match a.lifecycle.measured_file_ownership_release {
        MeasuredFileOwnershipReleaseV2::DropCompleted => q(&mut o, "drop_completed")?,
        MeasuredFileOwnershipReleaseV2::NotAcquired => q(&mut o, "not_acquired")?,
    }
    o.push_str("},\"first_causal_failure\":");
    match &a.first_causal_failure {
        Some(f) => failure_json(&mut o, f)?,
        None => o.push_str("null"),
    }
    o.push_str(",\"cleanup_failures\":[");
    for (i, f) in a.cleanup_failures.iter().enumerate() {
        if i > 0 {
            o.push(',')
        }
        failure_json(&mut o, f)?
    }
    o.push_str("],\"tracefs\":{\"state\":\"not_collected\",\"reason\":");
    q(&mut o, a.tracefs.reason)?;
    o.push_str("},\"classification\":");
    q(
        &mut o,
        match a.classification {
            ArtifactClassificationV1::PreflightSubsetReady => "preflight_subset_ready",
            ArtifactClassificationV1::Blocked => "blocked",
            ArtifactClassificationV1::Invalid => "invalid",
        },
    )?;
    o.push_str(",\"reasons\":[");
    for (i, r) in a.reasons.iter().enumerate() {
        if i > 0 {
            o.push(',')
        }
        q(&mut o, r)?
    }
    o.push_str("]}\n");
    Ok(o.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_validation_is_preopen_and_path_free() {
        let p = Path::new("/secret/path");
        for (r, b, i, e) in [
            (
                "x",
                "ok",
                "id",
                RequestFailureReasonV1::InvalidRepositoryRevision,
            ),
            (
                "1111111111111111111111111111111111111111",
                "bad/path",
                "id",
                RequestFailureReasonV1::InvalidBuildIdentity,
            ),
            (
                "1111111111111111111111111111111111111111",
                "ok",
                "bad..id",
                RequestFailureReasonV1::InvalidMeasuredFileIdentity,
            ),
        ] {
            let q = TargetPreflightRequest {
                repository_revision: r,
                build_identity: b,
                expected_platform: TargetPlatformV1::Fedora44Linux,
                expected_architecture: TargetArchitectureV1::X86_64,
                measured_file_path: p,
                measured_file_identity: i,
            };
            assert_eq!(validate(&q), Err(e));
            let x = invalid(e);
            assert!(x.artifact.is_none() && x.serialized_bytes.is_none());
            assert!(!format!("{x:?}").contains("secret"));
        }
    }
    #[test]
    fn escaping_rejects_non_ascii() {
        let mut s = String::new();
        assert!(q(&mut s, "é").is_err());
        let mut s = String::new();
        q(&mut s, "a\n\"\\").unwrap();
        assert_eq!(s, "\"a\\n\\\"\\\\\"");
    }
    #[test]
    fn io_fallback_is_other() {
        assert_eq!(map_io(io::ErrorKind::Other), IoErrorKindV1::Other);
        assert_eq!(io_name(io::ErrorKind::Other), "other");
        // These schema values are retained even though their std variants are unstable on 1.89.
        assert_eq!(
            match IoErrorKindV1::FilesystemLoop {
                IoErrorKindV1::FilesystemLoop => "filesystem_loop",
                _ => unreachable!(),
            },
            "filesystem_loop"
        );
        assert_eq!(
            match IoErrorKindV1::InProgress {
                IoErrorKindV1::InProgress => "in_progress",
                _ => unreachable!(),
            },
            "in_progress"
        );
    }

    #[test]
    fn every_closed_wrapper_and_nested_reason_has_an_exact_encoding() {
        let unavailable = [
            UnavailableReason::Interface(7),
            UnavailableReason::MissingStatxSize,
            UnavailableReason::NotFound,
            UnavailableReason::Unsupported,
            UnavailableReason::StatxOnlyAfterFstat,
        ];
        for value in unavailable {
            let mut json = String::new();
            PreflightOutcomeV1::Outcome(Outcome::<i32>::Unavailable(value))
                .json(&mut json)
                .unwrap();
            assert!(json.starts_with("{\"unavailable\":"));
        }
        let overflow = [
            OverflowReason::Arithmetic,
            OverflowReason::FileSize,
            OverflowReason::NumericField,
            OverflowReason::PerfScaling,
            OverflowReason::PerfErrno(9),
        ];
        for value in overflow {
            let mut json = String::new();
            PreflightOutcomeV1::Outcome(Outcome::<i32>::Overflow(value))
                .json(&mut json)
                .unwrap();
            assert!(json.starts_with("{\"overflow\":"));
        }
        let parse = [
            ParseReason::NonAscii,
            ParseReason::LineCount,
            ParseReason::TokenCount,
            ParseReason::MalformedLine,
            ParseReason::MissingField,
            ParseReason::DuplicateField,
            ParseReason::SignedValue,
            ParseReason::InvalidNumber,
            ParseReason::InvalidUnit,
            ParseReason::TrailingToken,
        ];
        for value in parse {
            let mut json = String::new();
            PreflightOutcomeV1::Outcome(Outcome::<i32>::Error(ErrorReason::Parse(value)))
                .json(&mut json)
                .unwrap();
            assert!(json.contains("\"kind\":\"parse\""));
        }
        let errors = [
            ErrorReason::Errno(1),
            ErrorReason::InvalidFraction,
            ErrorReason::NegativeCounter,
            ErrorReason::NegativeFileSize,
            ErrorReason::Io(io::ErrorKind::NotFound),
            ErrorReason::InvalidUtf8,
            ErrorReason::PerfShortRead(3),
            ErrorReason::PerfInvalidTime,
            ErrorReason::PerfDecrease,
            ErrorReason::PerfLifecycle,
            ErrorReason::PerfCleanup(4),
            ErrorReason::PerfUnexpectedReturn(5),
            ErrorReason::PerfCleanupUnexpected(6),
            ErrorReason::PerfEventMismatch {
                expected: PerfEvent::CpuCycles,
                actual: PerfEvent::Instructions,
            },
            ErrorReason::MissingFileCapability,
        ];
        for value in errors {
            let mut json = String::new();
            PreflightOutcomeV1::Outcome(Outcome::<i32>::Error(value))
                .json(&mut json)
                .unwrap();
            assert!(json.starts_with("{\"error\":"));
        }
        for outcome in [
            PreflightOutcomeV1::Outcome(Outcome::Success(1_i32)),
            PreflightOutcomeV1::Outcome(Outcome::Permission(13)),
            PreflightOutcomeV1::NotAttempted {
                failure_id: "failure-0001".into(),
            },
        ] {
            let mut json = String::new();
            outcome.json(&mut json).unwrap();
            assert!(!json.is_empty());
        }
    }
    use std::os::fd::{AsRawFd, RawFd};
    struct FakeFd;
    impl AsRawFd for FakeFd {
        fn as_raw_fd(&self) -> RawFd {
            panic!("descriptor must remain borrowed and unread by synthetic boundary")
        }
    }
    struct Synthetic;
    impl CaptureBoundary for Synthetic {
        type PerfOwner = PerfEvent;
        fn realtime(&mut self) -> Outcome<i128> {
            Outcome::Success(1_000_000_000)
        }
        fn monotonic_raw(&mut self) -> Outcome<i128> {
            Outcome::Success(2_000_000_000)
        }
        fn process_rusage(&mut self) -> Outcome<ResourceUsage> {
            Outcome::Success(ResourceUsage {
                user_nanoseconds: 100,
                system_nanoseconds: 50,
                maximum_resident_bytes: 8192,
                minor_faults: 2,
                major_faults: 0,
                input_blocks: 0,
                output_blocks: 0,
                voluntary_context_switches: 1,
                involuntary_context_switches: 0,
            })
        }
        fn thread_rusage(&mut self) -> Outcome<ResourceUsage> {
            Outcome::Success(ResourceUsage {
                user_nanoseconds: 80,
                system_nanoseconds: 40,
                maximum_resident_bytes: 8192,
                minor_faults: 1,
                major_faults: 0,
                input_blocks: 0,
                output_blocks: 0,
                voluntary_context_switches: 1,
                involuntary_context_switches: 0,
            })
        }
        fn statm(&mut self) -> Outcome<Statm> {
            Outcome::Success(Statm {
                size: 10,
                resident: 2,
                shared: 1,
                text: 1,
                lib: 0,
                data: 3,
                dt: 0,
            })
        }
        fn status(&mut self) -> Outcome<StatusMemory> {
            Outcome::Success(StatusMemory {
                resident_bytes: 8192,
                high_water_bytes: 12288,
            })
        }
        fn process_io(&mut self) -> Outcome<ProcessIo> {
            Outcome::Success(ProcessIo {
                rchar: 100,
                wchar: 0,
                syscr: 1,
                syscw: 0,
                read_bytes: 0,
                write_bytes: 0,
                cancelled_write_bytes: 0,
            })
        }
        fn file_length(&mut self, _: MeasuredFileReference<'_>) -> Outcome<FileLength> {
            Outcome::Success(FileLength {
                bytes: 4096,
                source: FileLengthSource::Statx,
                statx_only_fields: Ok(()),
            })
        }
        fn open_perf(&mut self, e: PerfEvent) -> Outcome<PerfEvent> {
            Outcome::Success(e)
        }
        fn stop_perf(&mut self, _: &mut PerfEvent, e: PerfEvent) -> Outcome<PerfCounter> {
            let n = match e {
                PerfEvent::CpuCycles => 10,
                PerfEvent::Instructions => 8,
                PerfEvent::PageFaults => 1,
                PerfEvent::ContextSwitches => 2,
            };
            Outcome::Success(PerfCounter {
                event: e,
                raw_count: n,
                time_enabled_ns: 10,
                time_running_ns: 10,
                multiplexed: false,
                scaled_count: Outcome::Success(n),
            })
        }
        fn cleanup_perf(&mut self, _: PerfEvent, _: PerfEvent) -> Outcome<()> {
            Outcome::Success(())
        }
    }
    impl ExecutionBoundary for Synthetic {
        fn resolution(&mut self, _: Clock) -> Outcome<i128> {
            Outcome::Success(1)
        }
    }
    fn fictional() -> TargetPreflightArtifactV2 {
        let r = TargetPreflightRequest {
            repository_revision: "1111111111111111111111111111111111111111",
            build_identity: "fictional-build-01",
            expected_platform: TargetPlatformV1::Fedora44Linux,
            expected_architecture: TargetArchitectureV1::X86_64,
            measured_file_path: Path::new("transient-do-not-retain"),
            measured_file_identity: "measured-file-alpha",
        };
        mark_drop_completed(execute_with_boundary(&r, &FakeFd, &mut Synthetic))
    }
    fn request() -> TargetPreflightRequest<'static> {
        TargetPreflightRequest {
            repository_revision: "1111111111111111111111111111111111111111",
            build_identity: "fictional-build-01",
            expected_platform: TargetPlatformV1::Fedora44Linux,
            expected_architecture: TargetArchitectureV1::X86_64,
            measured_file_path: Path::new("transient-do-not-retain"),
            measured_file_identity: "measured-file-alpha",
        }
    }
    #[test]
    fn metadata_failure_preserves_open_success_and_file_phase() {
        let artifact = mark_drop_completed(build_metadata_failure(
            &request(),
            io::Error::from(io::ErrorKind::Other),
        ));
        assert_eq!(
            artifact.measured_file.open,
            PreflightOutcomeV1::Outcome(Outcome::Success(()))
        );
        assert!(matches!(
            artifact.measured_file.regular_file,
            PreflightOutcomeV1::Outcome(Outcome::Error(ErrorReason::Io(_)))
        ));
        let failure = artifact.first_causal_failure.as_ref().unwrap();
        assert_eq!(failure.phase, FailurePhaseV1::MeasuredFileRegularFile);
        assert_eq!(failure.source, FailureSourceV1::MeasuredFile);
        assert_eq!(
            artifact.lifecycle.phases,
            [
                LifecyclePhaseV1::RequestValidated,
                LifecyclePhaseV1::FileOpened,
                LifecyclePhaseV1::OwnershipReleased
            ]
        );
        let mut sink = Sink::default();
        let execution = finish_artifact(artifact, &mut sink);
        assert_eq!(
            execution.disposition,
            TargetPreflightCallDispositionV1::Completed
        );
        assert_eq!(
            execution.artifact.unwrap().classification,
            ArtifactClassificationV1::Invalid
        );
    }

    #[test]
    fn nonregular_execution_is_retained_as_invalid_after_drop() {
        let artifact = mark_drop_completed(build_nonregular(&request()));
        let mut sink = Sink::default();
        let execution = finish_artifact(artifact, &mut sink);
        assert_eq!(
            execution.disposition,
            TargetPreflightCallDispositionV1::Completed
        );
        let retained = execution.artifact.unwrap();
        assert_eq!(retained.classification, ArtifactClassificationV1::Invalid);
        assert_eq!(
            retained.lifecycle.measured_file_ownership_release,
            MeasuredFileOwnershipReleaseV2::DropCompleted
        );
        assert!(
            std::str::from_utf8(execution.serialized_bytes.as_ref().unwrap())
                .unwrap()
                .contains("not_regular_file")
        );
    }
    #[test]
    fn supported_target_and_open_failure_are_exact() {
        let artifact = build_open_failure(&request(), io::Error::from_raw_os_error(2));
        assert_eq!(
            artifact.platform.disposition,
            PlatformDispositionV2::ProspectiveFedora44Linux
        );
        assert_eq!(
            artifact.architecture.observed,
            ArchitectureObservationV2::X86_64
        );
        assert_eq!(
            artifact.lifecycle.measured_file_ownership_release,
            MeasuredFileOwnershipReleaseV2::NotAcquired
        );
        let bytes = serialize(&artifact).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("\"measured_file_ownership_release\":\"not_acquired\""));
        assert!(text.contains("\"phase\":\"measured_file_open\""));
        assert!(!text.contains("\"file_opened\""));
        assert!(!text.contains("unsupported_target_os"));
        assert!(!text.contains("\"observed\":\"unsupported\""));
    }
    #[test]
    fn serialization_revalidates_schema_ids_and_ownership() {
        let mut artifact = fictional();
        artifact.schema = "EXP-0001-R33/target-preflight-artifact-v1";
        assert_eq!(serialize(&artifact), Err(()));
        let mut sink = Sink::default();
        let execution = finish_artifact(artifact, &mut sink);
        assert_eq!(
            execution.disposition,
            TargetPreflightCallDispositionV1::SerializationFailed
        );
        assert!(sink.bytes.is_empty());
        assert_eq!(sink.flushes, 0);

        let mut artifact = fictional();
        artifact.lifecycle.measured_file_ownership_release =
            MeasuredFileOwnershipReleaseV2::NotAcquired;
        assert_eq!(serialize(&artifact), Err(()));

        let mut artifact = fictional();
        artifact.cleanup_failures.push(FailureObjectV1 {
            id: "failure-0002".into(),
            phase: FailurePhaseV1::PerfCleanup,
            source: FailureSourceV1::Perf(PerfEvent::CpuCycles),
            detail: FailureDetailV1::Error(ErrorReasonV1::PerfCleanup(5)),
        });
        assert_eq!(serialize(&artifact), Err(()));
    }
    #[test]
    fn serialization_rejects_a_pre_drop_artifact() {
        let mut boundary = Synthetic;
        let draft = execute_with_boundary(&request(), &FakeFd, &mut boundary);
        assert_eq!(serialize(&draft), Err(()));
        assert!(serialize(&mark_drop_completed(draft)).is_ok());
    }

    #[test]
    fn complete_public_artifact_contradictions_fail_closed() {
        fn rejected(mutator: impl FnOnce(&mut TargetPreflightArtifactV2)) {
            let mut artifact = fictional();
            mutator(&mut artifact);
            assert_eq!(serialize(&artifact), Err(()));
        }
        rejected(|a| a.repository_revision = "A".repeat(40));
        rejected(|a| a.build_identity = "x".repeat(129));
        rejected(|a| a.build_identity = "bad/path".into());
        rejected(|a| a.measured_file.identity = "Bad".into());
        rejected(|a| a.measured_file.length.identity = SourceIdentityV1::Realtime);
        rejected(|a| a.sources.realtime.scope = ScopeV1::Process);
        rejected(|a| a.sources.status.unit = UnitV1::Nanoseconds);
        rejected(|a| a.perf_events.swap(0, 1));
        rejected(|a| a.perf_events[0].scope = ScopeV1::Process);
        rejected(|a| a.perf_events[0].unit = UnitV1::Bytes);
        rejected(|a| {
            if let PreflightOutcomeV1::Outcome(Outcome::Success(counter)) =
                &mut a.perf_events[0].stop_read
            {
                counter.event = PerfEvent::Instructions
            }
        });
        rejected(|a| a.lifecycle.phases.swap(1, 2));
        rejected(|a| {
            a.lifecycle.phases.pop();
        });
        rejected(|a| a.classification = ArtifactClassificationV1::Blocked);
        rejected(|a| a.tracefs.reason = "fabricated");
        rejected(|a| a.reasons.push("open text is closed".into()));
        rejected(|a| {
            a.first_causal_failure = Some(FailureObjectV1 {
                id: "failure-0001".into(),
                phase: FailurePhaseV1::PlatformValidation,
                source: FailureSourceV1::Platform,
                detail: FailureDetailV1::InvalidState(InvalidStateReasonV1::PlatformMismatch),
            })
        });
        rejected(|a| a.perf_events[0].open = skipped("failure-9999"));
    }
    #[test]
    fn exact_r33_vector_and_repeated_output() {
        let bytes = serialize(&fictional()).unwrap();
        assert_eq!(bytes, serialize(&fictional()).unwrap());
        let authority = include_str!(
            "../../../../../docs/experiments/EXP-0001/R33-R32-ADAPTER-CLOSURE-AND-TARGET-PREFLIGHT-DECISION.md"
        );
        let marker = "```json\n{\"schema\":\"EXP-0001-R33/target-preflight-artifact-v1\"";
        let start = authority.find(marker).unwrap() + "```json\n".len();
        let end = start + authority[start..].find("```\n").unwrap();
        let expected = authority[start..end].replace(
            "EXP-0001-R33/target-preflight-artifact-v1",
            TARGET_PREFLIGHT_ARTIFACT_SCHEMA_V2,
        );
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), expected);
        assert!(
            !std::str::from_utf8(&bytes)
                .unwrap()
                .contains("transient-do-not-retain")
        );
    }
    #[derive(Default)]
    struct Sink {
        bytes: Vec<u8>,
        flushes: usize,
        write_error: Option<io::Error>,
        flush_error: Option<io::Error>,
        partial_before_error: bool,
    }
    impl TargetPreflightRetention for Sink {
        fn write_all(&mut self, b: &[u8]) -> io::Result<()> {
            if let Some(e) = self.write_error.take() {
                if self.partial_before_error {
                    self.bytes.extend_from_slice(&b[..b.len().min(7)]);
                }
                return Err(e);
            }
            self.bytes.extend_from_slice(b);
            Ok(())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if let Some(e) = self.flush_error.take() {
                Err(e)
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn retention_is_separate_and_ordered() {
        let a = fictional();
        let mut sink = Sink::default();
        let x = finish_artifact(a.clone(), &mut sink);
        assert_eq!(x.disposition, TargetPreflightCallDispositionV1::Completed);
        assert_eq!(sink.flushes, 1);
        assert_eq!(x.artifact.unwrap(), a);
        let mut sink = Sink {
            write_error: Some(io::Error::from_raw_os_error(5)),
            partial_before_error: true,
            ..Sink::default()
        };
        let x = finish_artifact(a.clone(), &mut sink);
        assert_eq!(sink.flushes, 0);
        assert_eq!(sink.bytes.len(), 7);
        assert!(matches!(
            x.retention,
            RetentionOutcomeV1::IoFailure {
                operation: RetentionOperationV1::WriteAll,
                error: RetentionIoErrorV1 {
                    raw_os_error: Some(5),
                    ..
                }
            }
        ));
        let mut sink = Sink {
            write_error: Some(io::Error::from(io::ErrorKind::WriteZero)),
            ..Sink::default()
        };
        let x = finish_artifact(a.clone(), &mut sink);
        assert!(sink.bytes.is_empty());
        assert_eq!(sink.flushes, 0);
        assert!(matches!(
            x.retention,
            RetentionOutcomeV1::IoFailure {
                operation: RetentionOperationV1::WriteAll,
                error: RetentionIoErrorV1 {
                    kind: IoErrorKindV1::WriteZero,
                    ..
                }
            }
        ));
        let mut sink = Sink {
            flush_error: Some(io::Error::from_raw_os_error(13)),
            ..Sink::default()
        };
        let x = finish_artifact(a, &mut sink);
        assert_eq!(sink.flushes, 1);
        assert!(matches!(
            x.retention,
            RetentionOutcomeV1::IoFailure {
                operation: RetentionOperationV1::Flush,
                ..
            }
        ));
    }
}
