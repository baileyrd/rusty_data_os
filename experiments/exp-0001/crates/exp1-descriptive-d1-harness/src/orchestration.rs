//! R31's deterministic, non-live observation orchestrator.

use crate::linux_capture::{
    FileLength, Outcome, PerfCounter, PerfEvent, ProcessIo, ResourceUsage, Statm, StatusMemory,
};

pub const TRACE_NOT_COLLECTED_REASON: &str =
    "R31 first descriptive B1/D1 cell deliberately did not invoke tracefs";
pub const TRACE_UNSUPPORTED_REASON: &str =
    "R31 target preflight established that tracefs cannot supply this channel";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceChannel {
    Syscall,
    Scheduler,
    BlockIo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TraceMissingState {
    NotCollected,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceMissing {
    pub channel: TraceChannel,
    pub state: TraceMissingState,
    pub reason: &'static str,
    /// Required, retained evidence for `unsupported`; forbidden otherwise.
    pub preflight_evidence: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationPlan {
    pub cell_identity: String,
    pub observation_identity: String,
    pub subject_identity: String,
    pub measured_thread_identity: String,
    pub sources: SourceList,
    pub tracefs: [TraceMissing; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceList {
    pub realtime: bool,
    pub monotonic_raw: bool,
    pub process_rusage: bool,
    pub thread_rusage: bool,
    pub statm: bool,
    pub status: bool,
    pub process_io: bool,
    pub file_length: bool,
    pub perf: [PerfEvent; 4],
}

impl SourceList {
    pub const R31: Self = Self {
        realtime: true,
        monotonic_raw: true,
        process_rusage: true,
        thread_rusage: true,
        statm: true,
        status: true,
        process_io: true,
        file_length: true,
        perf: [
            PerfEvent::CpuCycles,
            PerfEvent::Instructions,
            PerfEvent::PageFaults,
            PerfEvent::ContextSwitches,
        ],
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeasuredFileIdentity<'a> {
    pub identity: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Created,
    BeforeCaptured,
    CountersArmed,
    Measuring,
    ActionCompleted,
    CountersStopped,
    AfterCaptured,
    Cleaned,
    Complete,
    Failed,
    CleanedAfterFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lifecycle {
    state: LifecycleState,
    pub ledger: Vec<LifecycleState>,
}

impl Lifecycle {
    pub fn new() -> Self {
        Self {
            state: LifecycleState::Created,
            ledger: vec![LifecycleState::Created],
        }
    }
    pub fn state(&self) -> LifecycleState {
        self.state
    }
    pub fn transition(&mut self, next: LifecycleState) -> Result<(), Failure> {
        use LifecycleState::*;
        let legal = matches!(
            (self.state, next),
            (Created, BeforeCaptured)
                | (BeforeCaptured, CountersArmed)
                | (CountersArmed, Measuring)
                | (Measuring, ActionCompleted)
                | (ActionCompleted, CountersStopped)
                | (CountersStopped, AfterCaptured)
                | (AfterCaptured, Cleaned)
                | (Cleaned, Complete)
                | (
                    Created
                        | BeforeCaptured
                        | CountersArmed
                        | Measuring
                        | ActionCompleted
                        | CountersStopped
                        | AfterCaptured,
                    Failed
                )
                | (Failed, CleanedAfterFailure)
        );
        if !legal {
            return Err(Failure::InvalidTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        self.ledger.push(next);
        Ok(())
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Plan,
    BeforeRealtime,
    BeforeProcessRusage,
    BeforeThreadRusage,
    BeforeStatm,
    BeforeStatus,
    BeforeProcessIo,
    BeforeFileLength,
    PerfOpen(PerfEvent),
    MonotonicStart,
    Action,
    MonotonicEnd,
    PerfStop(PerfEvent),
    AfterFileLength,
    AfterStatm,
    AfterStatus,
    AfterProcessIo,
    AfterProcessRusage,
    AfterThreadRusage,
    AfterRealtime,
    Elapsed,
    Cleanup(PerfEvent),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Failure {
    InvalidPlan(&'static str),
    Source {
        phase: Phase,
        outcome: Outcome<()>,
    },
    Action(String),
    MonotonicReversal {
        start: i128,
        end: i128,
    },
    ElapsedOverflow,
    InvalidTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pair<T> {
    pub before: Option<Outcome<T>>,
    pub after: Option<Outcome<T>>,
}
impl<T> Default for Pair<T> {
    fn default() -> Self {
        Self {
            before: None,
            after: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialObservation {
    pub plan: ObservationPlan,
    pub measured_file_identity: String,
    pub realtime_ns: Pair<i128>,
    pub monotonic_start_ns: Option<Outcome<i128>>,
    pub monotonic_end_ns: Option<Outcome<i128>>,
    pub elapsed_ns: Option<i128>,
    pub process_rusage: Pair<ResourceUsage>,
    pub thread_rusage: Pair<ResourceUsage>,
    pub statm: Pair<Statm>,
    pub status: Pair<StatusMemory>,
    pub process_io: Pair<ProcessIo>,
    pub file_length: Pair<FileLength>,
    pub perf: [Option<Outcome<PerfCounter>>; 4],
    pub action: Option<Result<(), String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteObservation {
    pub observation: PartialObservation,
    pub ledger: Vec<LifecycleState>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidObservation {
    pub observation: PartialObservation,
    pub ledger: Vec<LifecycleState>,
    pub primary_failure: Failure,
    pub cleanup_failures: Vec<Failure>,
    pub terminal: LifecycleState,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationOutcome {
    Complete(CompleteObservation),
    Invalid(InvalidObservation),
}

pub trait CaptureBoundary {
    type PerfOwner;
    fn realtime(&mut self) -> Outcome<i128>;
    fn monotonic_raw(&mut self) -> Outcome<i128>;
    fn process_rusage(&mut self) -> Outcome<ResourceUsage>;
    fn thread_rusage(&mut self) -> Outcome<ResourceUsage>;
    fn statm(&mut self) -> Outcome<Statm>;
    fn status(&mut self) -> Outcome<StatusMemory>;
    fn process_io(&mut self) -> Outcome<ProcessIo>;
    fn file_length(&mut self, file: MeasuredFileIdentity<'_>) -> Outcome<FileLength>;
    fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner>;
    fn stop_perf(&mut self, owner: &mut Self::PerfOwner, event: PerfEvent) -> Outcome<PerfCounter>;
    fn cleanup_perf(&mut self, owner: Self::PerfOwner, event: PerfEvent) -> Outcome<()>;
}

pub trait MeasuredAction {
    fn invoke(&mut self) -> Result<(), String>;
}

fn failure<T>(phase: Phase, value: &Outcome<T>) -> Failure {
    let outcome = match value {
        Outcome::Success(_) => unreachable!(),
        Outcome::Unavailable(v) => Outcome::Unavailable(*v),
        Outcome::Permission(v) => Outcome::Permission(*v),
        Outcome::Overflow(v) => Outcome::Overflow(*v),
        Outcome::Error(v) => Outcome::Error(v.clone()),
    };
    Failure::Source { phase, outcome }
}

fn validate(plan: &ObservationPlan) -> Result<(), Failure> {
    if plan.cell_identity.is_empty()
        || plan.observation_identity.is_empty()
        || plan.subject_identity.is_empty()
        || plan.measured_thread_identity.is_empty()
    {
        return Err(Failure::InvalidPlan("identities must be nonempty"));
    }
    if plan.sources != SourceList::R31 {
        return Err(Failure::InvalidPlan(
            "source list is not the frozen R31 list",
        ));
    }
    let channels = [
        TraceChannel::Syscall,
        TraceChannel::Scheduler,
        TraceChannel::BlockIo,
    ];
    for (entry, channel) in plan.tracefs.iter().zip(channels) {
        if entry.channel != channel {
            return Err(Failure::InvalidPlan(
                "tracefs channels are not in frozen order",
            ));
        }
        match entry.state {
            TraceMissingState::NotCollected
                if entry.reason == TRACE_NOT_COLLECTED_REASON
                    && entry.preflight_evidence.is_none() => {}
            TraceMissingState::Unsupported
                if entry.reason == TRACE_UNSUPPORTED_REASON
                    && entry
                        .preflight_evidence
                        .as_ref()
                        .is_some_and(|v| !v.is_empty()) => {}
            TraceMissingState::Unsupported => {
                return Err(Failure::InvalidPlan(
                    "unsupported tracefs requires retained preflight evidence and exact reason",
                ));
            }
            TraceMissingState::NotCollected => {
                return Err(Failure::InvalidPlan(
                    "not_collected tracefs requires exact reason and no preflight evidence",
                ));
            }
        }
    }
    Ok(())
}

/// Executes only injected calls. Each perf owner is cleaned once, in reverse acquisition order.
pub fn observe<B: CaptureBoundary, A: MeasuredAction>(
    plan: &ObservationPlan,
    file: MeasuredFileIdentity<'_>,
    boundary: &mut B,
    action: &mut A,
) -> ObservationOutcome {
    let mut life = Lifecycle::new();
    let mut data = PartialObservation {
        plan: plan.clone(),
        measured_file_identity: file.identity.to_owned(),
        realtime_ns: Pair::default(),
        monotonic_start_ns: None,
        monotonic_end_ns: None,
        elapsed_ns: None,
        process_rusage: Pair::default(),
        thread_rusage: Pair::default(),
        statm: Pair::default(),
        status: Pair::default(),
        process_io: Pair::default(),
        file_length: Pair::default(),
        perf: [None, None, None, None],
        action: None,
    };
    let mut owners: Vec<(PerfEvent, B::PerfOwner)> = Vec::new();
    let mut primary = validate(plan).err();
    macro_rules! required {
        ($slot:expr, $call:expr, $phase:expr) => {{
            if primary.is_none() {
                let value = $call;
                let bad = !matches!(value, Outcome::Success(_));
                $slot = Some(value);
                if bad {
                    primary = Some(failure($phase, $slot.as_ref().unwrap()));
                }
            }
        }};
    }
    if primary.is_none() {
        required!(
            data.realtime_ns.before,
            boundary.realtime(),
            Phase::BeforeRealtime
        );
        required!(
            data.process_rusage.before,
            boundary.process_rusage(),
            Phase::BeforeProcessRusage
        );
        required!(
            data.thread_rusage.before,
            boundary.thread_rusage(),
            Phase::BeforeThreadRusage
        );
        required!(data.statm.before, boundary.statm(), Phase::BeforeStatm);
        required!(data.status.before, boundary.status(), Phase::BeforeStatus);
        required!(
            data.process_io.before,
            boundary.process_io(),
            Phase::BeforeProcessIo
        );
        required!(
            data.file_length.before,
            boundary.file_length(file),
            Phase::BeforeFileLength
        );
        if primary.is_none() {
            let _ = life.transition(LifecycleState::BeforeCaptured);
        }
    }
    if primary.is_none() {
        for (index, event) in SourceList::R31.perf.into_iter().enumerate() {
            match boundary.open_perf(event) {
                Outcome::Success(owner) => owners.push((event, owner)),
                value @ (Outcome::Unavailable(_) | Outcome::Permission(_)) => {
                    data.perf[index] = Some(value.map_type_for_orchestration())
                }
                value => {
                    data.perf[index] = Some(value.map_type_for_orchestration());
                    primary = Some(failure(
                        Phase::PerfOpen(event),
                        data.perf[index].as_ref().unwrap(),
                    ));
                    break;
                }
            }
        }
        if primary.is_none() {
            let _ = life.transition(LifecycleState::CountersArmed);
        }
    }
    if primary.is_none() {
        required!(
            data.monotonic_start_ns,
            boundary.monotonic_raw(),
            Phase::MonotonicStart
        );
        if primary.is_none() {
            let _ = life.transition(LifecycleState::Measuring);
        }
    }
    if primary.is_none() {
        let result = action.invoke();
        data.action = Some(result.clone());
        if let Err(error) = result {
            primary = Some(Failure::Action(error));
        } else {
            let _ = life.transition(LifecycleState::ActionCompleted);
        }
    }
    if primary.is_none() {
        required!(
            data.monotonic_end_ns,
            boundary.monotonic_raw(),
            Phase::MonotonicEnd
        );
    }
    if primary.is_none() {
        for index in (0..owners.len()).rev() {
            let (event, owner) = &mut owners[index];
            let value = boundary.stop_perf(owner, *event);
            let bad = !matches!(value, Outcome::Success(_));
            let perf_index = SourceList::R31
                .perf
                .iter()
                .position(|v| v == event)
                .unwrap();
            data.perf[perf_index] = Some(value);
            if bad && primary.is_none() {
                primary = Some(failure(
                    Phase::PerfStop(*event),
                    data.perf[perf_index].as_ref().unwrap(),
                ));
            }
        }
        if primary.is_none() {
            let _ = life.transition(LifecycleState::CountersStopped);
        }
    }
    if primary.is_none() {
        required!(
            data.file_length.after,
            boundary.file_length(file),
            Phase::AfterFileLength
        );
        required!(data.statm.after, boundary.statm(), Phase::AfterStatm);
        required!(data.status.after, boundary.status(), Phase::AfterStatus);
        required!(
            data.process_io.after,
            boundary.process_io(),
            Phase::AfterProcessIo
        );
        required!(
            data.process_rusage.after,
            boundary.process_rusage(),
            Phase::AfterProcessRusage
        );
        required!(
            data.thread_rusage.after,
            boundary.thread_rusage(),
            Phase::AfterThreadRusage
        );
        required!(
            data.realtime_ns.after,
            boundary.realtime(),
            Phase::AfterRealtime
        );
        if primary.is_none() {
            let _ = life.transition(LifecycleState::AfterCaptured);
        }
    }
    if primary.is_none() {
        let start = match data.monotonic_start_ns {
            Some(Outcome::Success(v)) => v,
            _ => unreachable!(),
        };
        let end = match data.monotonic_end_ns {
            Some(Outcome::Success(v)) => v,
            _ => unreachable!(),
        };
        if end < start {
            primary = Some(Failure::MonotonicReversal { start, end });
        } else if let Some(value) = end.checked_sub(start) {
            data.elapsed_ns = Some(value);
        } else {
            primary = Some(Failure::ElapsedOverflow);
        }
    }
    if primary.is_some() && life.state() != LifecycleState::Failed {
        let _ = life.transition(LifecycleState::Failed);
    }
    let mut cleanup_failures = Vec::new();
    while let Some((event, owner)) = owners.pop() {
        let value = boundary.cleanup_perf(owner, event);
        if !matches!(value, Outcome::Success(())) {
            let item = failure(Phase::Cleanup(event), &value);
            if primary.is_none() {
                primary = Some(item.clone());
            } else {
                cleanup_failures.push(item);
            }
        }
    }
    if let Some(primary_failure) = primary {
        if life.state() != LifecycleState::Failed {
            let _ = life.transition(LifecycleState::Failed);
        }
        let _ = life.transition(LifecycleState::CleanedAfterFailure);
        ObservationOutcome::Invalid(InvalidObservation {
            observation: data,
            ledger: life.ledger,
            primary_failure,
            cleanup_failures,
            terminal: LifecycleState::CleanedAfterFailure,
        })
    } else {
        let _ = life.transition(LifecycleState::Cleaned);
        let _ = life.transition(LifecycleState::Complete);
        ObservationOutcome::Complete(CompleteObservation {
            observation: data,
            ledger: life.ledger,
        })
    }
}

trait MapOutcome<T> {
    fn map_type_for_orchestration<U>(self) -> Outcome<U>;
}
impl<T> MapOutcome<T> for Outcome<T> {
    fn map_type_for_orchestration<U>(self) -> Outcome<U> {
        match self {
            Outcome::Success(_) => unreachable!(),
            Outcome::Unavailable(v) => Outcome::Unavailable(v),
            Outcome::Permission(v) => Outcome::Permission(v),
            Outcome::Overflow(v) => Outcome::Overflow(v),
            Outcome::Error(v) => Outcome::Error(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linux_capture::{ErrorReason, FileLengthSource, OverflowReason, UnavailableReason};
    use std::collections::VecDeque;

    fn traces(state: TraceMissingState) -> [TraceMissing; 3] {
        [
            TraceChannel::Syscall,
            TraceChannel::Scheduler,
            TraceChannel::BlockIo,
        ]
        .map(|channel| TraceMissing {
            channel,
            state,
            reason: match state {
                TraceMissingState::NotCollected => TRACE_NOT_COLLECTED_REASON,
                TraceMissingState::Unsupported => TRACE_UNSUPPORTED_REASON,
            },
            preflight_evidence: (state == TraceMissingState::Unsupported)
                .then(|| "retained preflight #1".into()),
        })
    }
    fn plan() -> ObservationPlan {
        ObservationPlan {
            cell_identity: "B1/D1".into(),
            observation_identity: "o1".into(),
            subject_identity: "subject".into(),
            measured_thread_identity: "thread".into(),
            sources: SourceList::R31,
            tracefs: traces(TraceMissingState::NotCollected),
        }
    }
    fn usage() -> ResourceUsage {
        ResourceUsage {
            user_nanoseconds: 1,
            system_nanoseconds: 2,
            maximum_resident_bytes: 3,
            minor_faults: 4,
            major_faults: 5,
            input_blocks: 6,
            output_blocks: 7,
            voluntary_context_switches: 8,
            involuntary_context_switches: 9,
        }
    }
    fn counter(event: PerfEvent) -> PerfCounter {
        PerfCounter {
            event,
            raw_count: 10,
            time_enabled_ns: 20,
            time_running_ns: 20,
            multiplexed: false,
            scaled_count: Outcome::Success(10),
        }
    }

    #[derive(Default)]
    struct Synthetic {
        calls: Vec<String>,
        fail_call: Option<String>,
        cleanup_fail: Vec<PerfEvent>,
        next_clock: VecDeque<i128>,
    }
    impl Synthetic {
        fn mark(&mut self, call: String) -> bool {
            self.calls.push(call.clone());
            self.fail_call.as_ref() == Some(&call)
        }
    }
    impl CaptureBoundary for Synthetic {
        type PerfOwner = PerfEvent;
        fn realtime(&mut self) -> Outcome<i128> {
            let n = self.calls.iter().filter(|v| *v == "realtime").count();
            if self.mark("realtime".into()) && n == 0 {
                Outcome::Error(ErrorReason::Errno(1))
            } else {
                Outcome::Success(100 + n as i128)
            }
        }
        fn monotonic_raw(&mut self) -> Outcome<i128> {
            self.mark("monotonic".into());
            Outcome::Success(self.next_clock.pop_front().unwrap_or(10))
        }
        fn process_rusage(&mut self) -> Outcome<ResourceUsage> {
            if self.mark("process_rusage".into()) {
                Outcome::Error(ErrorReason::Errno(2))
            } else {
                Outcome::Success(usage())
            }
        }
        fn thread_rusage(&mut self) -> Outcome<ResourceUsage> {
            if self.mark("thread_rusage".into()) {
                Outcome::Error(ErrorReason::Errno(3))
            } else {
                Outcome::Success(usage())
            }
        }
        fn statm(&mut self) -> Outcome<Statm> {
            if self.mark("statm".into()) {
                Outcome::Error(ErrorReason::Errno(4))
            } else {
                Outcome::Success(Statm {
                    size: 1,
                    resident: 2,
                    shared: 3,
                    text: 4,
                    lib: 5,
                    data: 6,
                    dt: 7,
                })
            }
        }
        fn status(&mut self) -> Outcome<StatusMemory> {
            if self.mark("status".into()) {
                Outcome::Error(ErrorReason::Errno(5))
            } else {
                Outcome::Success(StatusMemory {
                    resident_bytes: 1,
                    high_water_bytes: 2,
                })
            }
        }
        fn process_io(&mut self) -> Outcome<ProcessIo> {
            if self.mark("io".into()) {
                Outcome::Error(ErrorReason::Errno(6))
            } else {
                Outcome::Success(ProcessIo {
                    rchar: 1,
                    wchar: 2,
                    syscr: 3,
                    syscw: 4,
                    read_bytes: 5,
                    write_bytes: 6,
                    cancelled_write_bytes: 7,
                })
            }
        }
        fn file_length(&mut self, file: MeasuredFileIdentity<'_>) -> Outcome<FileLength> {
            assert_eq!(file.identity, "fd-7");
            if self.mark("file".into()) {
                Outcome::Error(ErrorReason::Errno(7))
            } else {
                Outcome::Success(FileLength {
                    bytes: 8,
                    source: FileLengthSource::Statx,
                    statx_only_fields: Ok(()),
                })
            }
        }
        fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner> {
            let name = format!("open:{event:?}");
            if self.mark(name) {
                Outcome::Error(ErrorReason::Errno(8))
            } else {
                Outcome::Success(event)
            }
        }
        fn stop_perf(
            &mut self,
            _owner: &mut Self::PerfOwner,
            event: PerfEvent,
        ) -> Outcome<PerfCounter> {
            let name = format!("stop:{event:?}");
            if self.mark(name) {
                Outcome::Overflow(OverflowReason::PerfScaling)
            } else {
                Outcome::Success(counter(event))
            }
        }
        fn cleanup_perf(&mut self, _owner: Self::PerfOwner, event: PerfEvent) -> Outcome<()> {
            self.calls.push(format!("cleanup:{event:?}"));
            if self.cleanup_fail.contains(&event) {
                Outcome::Error(ErrorReason::PerfCleanup(9))
            } else {
                Outcome::Success(())
            }
        }
    }
    #[derive(Default)]
    struct Action {
        calls: usize,
        fail: bool,
    }
    impl MeasuredAction for Action {
        fn invoke(&mut self) -> Result<(), String> {
            self.calls += 1;
            if self.fail {
                Err("action failed".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn success_has_exact_order_complete_sources_and_once_action() {
        let mut boundary = Synthetic {
            next_clock: VecDeque::from([10, 15]),
            ..Synthetic::default()
        };
        let mut action = Action::default();
        let result = observe(
            &plan(),
            MeasuredFileIdentity { identity: "fd-7" },
            &mut boundary,
            &mut action,
        );
        let ObservationOutcome::Complete(done) = result else {
            panic!("must complete")
        };
        assert_eq!(action.calls, 1);
        assert_eq!(done.observation.elapsed_ns, Some(5));
        assert_eq!(
            done.ledger,
            [
                LifecycleState::Created,
                LifecycleState::BeforeCaptured,
                LifecycleState::CountersArmed,
                LifecycleState::Measuring,
                LifecycleState::ActionCompleted,
                LifecycleState::CountersStopped,
                LifecycleState::AfterCaptured,
                LifecycleState::Cleaned,
                LifecycleState::Complete
            ]
        );
        assert_eq!(
            boundary.calls,
            [
                "realtime",
                "process_rusage",
                "thread_rusage",
                "statm",
                "status",
                "io",
                "file",
                "open:CpuCycles",
                "open:Instructions",
                "open:PageFaults",
                "open:ContextSwitches",
                "monotonic",
                "monotonic",
                "stop:ContextSwitches",
                "stop:PageFaults",
                "stop:Instructions",
                "stop:CpuCycles",
                "file",
                "statm",
                "status",
                "io",
                "process_rusage",
                "thread_rusage",
                "realtime",
                "cleanup:ContextSwitches",
                "cleanup:PageFaults",
                "cleanup:Instructions",
                "cleanup:CpuCycles"
            ]
        );
        assert!(
            done.observation
                .perf
                .iter()
                .all(|v| matches!(v, Some(Outcome::Success(_))))
        );
    }

    #[test]
    fn lifecycle_accepts_only_the_frozen_paths() {
        let success = [
            LifecycleState::BeforeCaptured,
            LifecycleState::CountersArmed,
            LifecycleState::Measuring,
            LifecycleState::ActionCompleted,
            LifecycleState::CountersStopped,
            LifecycleState::AfterCaptured,
            LifecycleState::Cleaned,
            LifecycleState::Complete,
        ];
        let mut lifecycle = Lifecycle::new();
        for state in success {
            lifecycle.transition(state).unwrap();
        }
        assert!(lifecycle.transition(LifecycleState::Complete).is_err());
        for start in 0..7 {
            let mut lifecycle = Lifecycle::new();
            for state in success.iter().take(start) {
                lifecycle.transition(*state).unwrap();
            }
            lifecycle.transition(LifecycleState::Failed).unwrap();
            lifecycle
                .transition(LifecycleState::CleanedAfterFailure)
                .unwrap();
            assert!(lifecycle.transition(LifecycleState::Complete).is_err());
        }
        let mut lifecycle = Lifecycle::new();
        assert!(lifecycle.transition(LifecycleState::Measuring).is_err());
    }

    #[test]
    fn availability_is_valid_but_post_open_failure_is_invalid_and_cleanup_is_reverse() {
        struct Availability(Synthetic);
        impl CaptureBoundary for Availability {
            type PerfOwner = PerfEvent;
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
            fn file_length(&mut self, f: MeasuredFileIdentity<'_>) -> Outcome<FileLength> {
                self.0.file_length(f)
            }
            fn open_perf(&mut self, e: PerfEvent) -> Outcome<PerfEvent> {
                match e {
                    PerfEvent::CpuCycles => Outcome::Unavailable(UnavailableReason::Unsupported),
                    PerfEvent::Instructions => Outcome::Permission(13),
                    _ => self.0.open_perf(e),
                }
            }
            fn stop_perf(&mut self, o: &mut PerfEvent, e: PerfEvent) -> Outcome<PerfCounter> {
                self.0.stop_perf(o, e)
            }
            fn cleanup_perf(&mut self, o: PerfEvent, e: PerfEvent) -> Outcome<()> {
                self.0.cleanup_perf(o, e)
            }
        }
        let mut b = Availability(Synthetic {
            next_clock: VecDeque::from([1, 2]),
            ..Synthetic::default()
        });
        let mut a = Action::default();
        assert!(matches!(
            observe(
                &plan(),
                MeasuredFileIdentity { identity: "fd-7" },
                &mut b,
                &mut a
            ),
            ObservationOutcome::Complete(_)
        ));
        let mut b = Synthetic {
            fail_call: Some("stop:PageFaults".into()),
            next_clock: VecDeque::from([1, 2]),
            ..Synthetic::default()
        };
        let mut a = Action::default();
        let ObservationOutcome::Invalid(v) = observe(
            &plan(),
            MeasuredFileIdentity { identity: "fd-7" },
            &mut b,
            &mut a,
        ) else {
            panic!()
        };
        assert!(matches!(
            v.primary_failure,
            Failure::Source {
                phase: Phase::PerfStop(PerfEvent::PageFaults),
                ..
            }
        ));
        assert_eq!(
            &b.calls[b.calls.len() - 4..],
            [
                "cleanup:ContextSwitches",
                "cleanup:PageFaults",
                "cleanup:Instructions",
                "cleanup:CpuCycles"
            ]
        );
    }

    #[test]
    fn action_reversal_cleanup_failures_and_partial_values_fail_closed() {
        let mut b = Synthetic {
            next_clock: VecDeque::from([5, 4]),
            cleanup_fail: vec![PerfEvent::ContextSwitches, PerfEvent::Instructions],
            ..Synthetic::default()
        };
        let mut a = Action::default();
        let ObservationOutcome::Invalid(v) = observe(
            &plan(),
            MeasuredFileIdentity { identity: "fd-7" },
            &mut b,
            &mut a,
        ) else {
            panic!()
        };
        assert!(matches!(
            v.primary_failure,
            Failure::MonotonicReversal { .. }
        ));
        assert_eq!(
            v.cleanup_failures
                .iter()
                .map(|f| match f {
                    Failure::Source { phase, .. } => *phase,
                    _ => panic!(),
                })
                .collect::<Vec<_>>(),
            [
                Phase::Cleanup(PerfEvent::ContextSwitches),
                Phase::Cleanup(PerfEvent::Instructions)
            ]
        );
        assert_eq!(a.calls, 1);
        assert_eq!(v.terminal, LifecycleState::CleanedAfterFailure);
        assert!(v.observation.elapsed_ns.is_none());
        let mut b = Synthetic::default();
        let mut a = Action {
            fail: true,
            ..Action::default()
        };
        let _ = observe(
            &plan(),
            MeasuredFileIdentity { identity: "fd-7" },
            &mut b,
            &mut a,
        );
        assert_eq!(a.calls, 1);
    }

    #[test]
    fn required_phase_failure_stops_and_tracefs_evidence_is_exact() {
        for call in [
            "realtime",
            "process_rusage",
            "thread_rusage",
            "statm",
            "status",
            "io",
            "file",
        ] {
            let mut b = Synthetic {
                fail_call: Some(call.into()),
                ..Synthetic::default()
            };
            let mut a = Action::default();
            assert!(matches!(
                observe(
                    &plan(),
                    MeasuredFileIdentity { identity: "fd-7" },
                    &mut b,
                    &mut a
                ),
                ObservationOutcome::Invalid(_)
            ));
            assert_eq!(a.calls, 0);
        }
        let mut supported = plan();
        supported.tracefs = traces(TraceMissingState::Unsupported);
        assert!(validate(&supported).is_ok());
        supported.tracefs[1].preflight_evidence = None;
        assert!(validate(&supported).is_err());
        let mut wrong = plan();
        wrong.tracefs[0].reason = "similar but unauthorized";
        assert!(validate(&wrong).is_err());
    }
}
