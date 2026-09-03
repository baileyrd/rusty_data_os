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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceIdentity {
    Realtime,
    MonotonicRaw,
    ProcessRusage,
    ThreadRusage,
    Statm,
    Status,
    ProcessIo,
    FileLength,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceScope {
    Observation,
    Process,
    MeasuredThread,
    MeasuredFile,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unit {
    Nanoseconds,
    ResourceUsageFields,
    ProcfsFields,
    Bytes,
    PerfCounterFields,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourcePair<T> {
    pub identity: SourceIdentity,
    pub scope: SourceScope,
    pub unit: Unit,
    pub before: Option<Outcome<T>>,
    pub after: Option<Outcome<T>>,
}
impl<T> SourcePair<T> {
    fn new(identity: SourceIdentity, scope: SourceScope, unit: Unit) -> Self {
        Self {
            identity,
            scope,
            unit,
            before: None,
            after: None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerfObservation {
    pub event: PerfEvent,
    pub scope: SourceScope,
    pub unit: Unit,
    pub outcome: Option<Outcome<PerfCounter>>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupStatus {
    Successful,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialObservation {
    pub plan: ObservationPlan,
    pub measured_file_identity: String,
    pub realtime_ns: SourcePair<i128>,
    pub monotonic_raw_ns: SourcePair<i128>,
    pub elapsed_ns: Option<i128>,
    pub process_rusage: SourcePair<ResourceUsage>,
    pub thread_rusage: SourcePair<ResourceUsage>,
    pub statm: SourcePair<Statm>,
    pub status: SourcePair<StatusMemory>,
    pub process_io: SourcePair<ProcessIo>,
    pub file_length: SourcePair<FileLength>,
    pub perf: [PerfObservation; 4],
    pub action: Option<Result<(), String>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteObservation {
    pub observation: PartialObservation,
    pub ledger: Vec<LifecycleState>,
    pub cleanup: CleanupStatus,
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
    /// Testable transition boundary; implementations normally use the exact state machine.
    fn transition(
        &mut self,
        lifecycle: &mut Lifecycle,
        next: LifecycleState,
    ) -> Result<(), Failure> {
        lifecycle.transition(next)
    }
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
    for (entry, channel) in plan.tracefs.iter().zip([
        TraceChannel::Syscall,
        TraceChannel::Scheduler,
        TraceChannel::BlockIo,
    ]) {
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
        realtime_ns: SourcePair::new(
            SourceIdentity::Realtime,
            SourceScope::Observation,
            Unit::Nanoseconds,
        ),
        monotonic_raw_ns: SourcePair::new(
            SourceIdentity::MonotonicRaw,
            SourceScope::Observation,
            Unit::Nanoseconds,
        ),
        elapsed_ns: None,
        process_rusage: SourcePair::new(
            SourceIdentity::ProcessRusage,
            SourceScope::Process,
            Unit::ResourceUsageFields,
        ),
        thread_rusage: SourcePair::new(
            SourceIdentity::ThreadRusage,
            SourceScope::MeasuredThread,
            Unit::ResourceUsageFields,
        ),
        statm: SourcePair::new(
            SourceIdentity::Statm,
            SourceScope::Process,
            Unit::ProcfsFields,
        ),
        status: SourcePair::new(SourceIdentity::Status, SourceScope::Process, Unit::Bytes),
        process_io: SourcePair::new(
            SourceIdentity::ProcessIo,
            SourceScope::Process,
            Unit::ProcfsFields,
        ),
        file_length: SourcePair::new(
            SourceIdentity::FileLength,
            SourceScope::MeasuredFile,
            Unit::Bytes,
        ),
        perf: SourceList::R31.perf.map(|event| PerfObservation {
            event,
            scope: SourceScope::MeasuredThread,
            unit: Unit::PerfCounterFields,
            outcome: None,
        }),
        action: None,
    };
    let mut owners: Vec<(PerfEvent, B::PerfOwner)> = Vec::new();
    let mut primary = validate(plan).err();
    let mut later_failures = Vec::new();
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
    macro_rules! transition {
        ($next:expr) => {{
            if primary.is_none() {
                if let Err(error) = boundary.transition(&mut life, $next) {
                    primary = Some(error);
                }
            }
        }};
    }

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
    transition!(LifecycleState::BeforeCaptured);

    if primary.is_none() {
        for (index, event) in SourceList::R31.perf.into_iter().enumerate() {
            match boundary.open_perf(event) {
                Outcome::Success(owner) => owners.push((event, owner)),
                value @ (Outcome::Unavailable(_) | Outcome::Permission(_)) => {
                    data.perf[index].outcome = Some(value.map_type_for_orchestration())
                }
                value => {
                    data.perf[index].outcome = Some(value.map_type_for_orchestration());
                    primary = Some(failure(
                        Phase::PerfOpen(event),
                        data.perf[index].outcome.as_ref().unwrap(),
                    ));
                    break;
                }
            }
        }
    }
    transition!(LifecycleState::CountersArmed);
    required!(
        data.monotonic_raw_ns.before,
        boundary.monotonic_raw(),
        Phase::MonotonicStart
    );
    transition!(LifecycleState::Measuring);
    if primary.is_none() {
        let result = action.invoke();
        data.action = Some(result.clone());
        if let Err(error) = result {
            primary = Some(Failure::Action(error));
        }
    }
    transition!(LifecycleState::ActionCompleted);
    required!(
        data.monotonic_raw_ns.after,
        boundary.monotonic_raw(),
        Phase::MonotonicEnd
    );
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
            data.perf[perf_index].outcome = Some(value);
            if bad && primary.is_none() {
                primary = Some(failure(
                    Phase::PerfStop(*event),
                    data.perf[perf_index].outcome.as_ref().unwrap(),
                ));
            }
        }
    }
    transition!(LifecycleState::CountersStopped);
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
    transition!(LifecycleState::AfterCaptured);
    if primary.is_none() {
        let start = match data.monotonic_raw_ns.before {
            Some(Outcome::Success(v)) => v,
            _ => unreachable!(),
        };
        let end = match data.monotonic_raw_ns.after {
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
    if primary.is_some()
        && life.state() != LifecycleState::Failed
        && let Err(error) = boundary.transition(&mut life, LifecycleState::Failed)
    {
        later_failures.push(error);
    }
    let mut cleanup_failures = later_failures;
    while let Some((event, owner)) = owners.pop() {
        let value = boundary.cleanup_perf(owner, event);
        if !matches!(value, Outcome::Success(())) {
            let item = failure(Phase::Cleanup(event), &value);
            if primary.is_none() {
                primary = Some(item);
            } else {
                cleanup_failures.push(item);
            }
        }
    }
    if primary.is_none()
        && let Err(error) = boundary.transition(&mut life, LifecycleState::Cleaned)
    {
        primary = Some(error);
    }
    if primary.is_none()
        && let Err(error) = boundary.transition(&mut life, LifecycleState::Complete)
    {
        primary = Some(error);
    }
    if let Some(primary_failure) = primary {
        if life.state() != LifecycleState::Failed
            && let Err(error) = boundary.transition(&mut life, LifecycleState::Failed)
        {
            cleanup_failures.push(error);
        }
        if life.state() == LifecycleState::Failed
            && let Err(error) = boundary.transition(&mut life, LifecycleState::CleanedAfterFailure)
        {
            cleanup_failures.push(error);
        }
        let terminal = life.state();
        ObservationOutcome::Invalid(InvalidObservation {
            observation: data,
            ledger: life.ledger,
            primary_failure,
            cleanup_failures,
            terminal,
        })
    } else {
        ObservationOutcome::Complete(CompleteObservation {
            observation: data,
            ledger: life.ledger,
            cleanup: CleanupStatus::Successful,
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
    use std::{cell::RefCell, collections::VecDeque, rc::Rc};

    fn traces(state: TraceMissingState) -> [TraceMissing; 3] {
        [
            TraceChannel::Syscall,
            TraceChannel::Scheduler,
            TraceChannel::BlockIo,
        ]
        .map(|channel| TraceMissing {
            channel,
            state,
            reason: if state == TraceMissingState::NotCollected {
                TRACE_NOT_COLLECTED_REASON
            } else {
                TRACE_UNSUPPORTED_REASON
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
    fn ledger() -> Rc<RefCell<Vec<String>>> {
        Rc::new(RefCell::new(Vec::new()))
    }

    #[derive(Clone, Copy)]
    enum PerfFault {
        Unavailable,
        Permission,
        Error,
        Overflow,
    }
    struct Synthetic {
        calls: Rc<RefCell<Vec<String>>>,
        fail_call: Option<String>,
        fail_occurrence: usize,
        perf_fault: Option<(PerfEvent, PerfFault)>,
        cleanup_fail: Vec<PerfEvent>,
        transition_fail: Option<LifecycleState>,
        next_clock: VecDeque<i128>,
    }
    impl Default for Synthetic {
        fn default() -> Self {
            Self {
                calls: ledger(),
                fail_call: None,
                fail_occurrence: 1,
                perf_fault: None,
                cleanup_fail: vec![],
                transition_fail: None,
                next_clock: VecDeque::from([10, 15]),
            }
        }
    }
    impl Synthetic {
        fn mark(&mut self, call: String) -> bool {
            self.calls.borrow_mut().push(call.clone());
            self.fail_call.as_ref() == Some(&call)
                && self.calls.borrow().iter().filter(|v| **v == call).count()
                    == self.fail_occurrence
        }
        fn ordinary<T>(&mut self, name: &str, value: T, errno: i32) -> Outcome<T> {
            if self.mark(name.into()) {
                Outcome::Error(ErrorReason::Errno(errno))
            } else {
                Outcome::Success(value)
            }
        }
    }
    impl CaptureBoundary for Synthetic {
        type PerfOwner = PerfEvent;
        fn realtime(&mut self) -> Outcome<i128> {
            self.ordinary("realtime", 100, 1)
        }
        fn monotonic_raw(&mut self) -> Outcome<i128> {
            self.calls.borrow_mut().push("monotonic".into());
            if self.fail_call.as_deref() == Some("monotonic")
                && self
                    .calls
                    .borrow()
                    .iter()
                    .filter(|v| v.as_str() == "monotonic")
                    .count()
                    == self.fail_occurrence
            {
                Outcome::Error(ErrorReason::Errno(2))
            } else {
                Outcome::Success(self.next_clock.pop_front().unwrap())
            }
        }
        fn process_rusage(&mut self) -> Outcome<ResourceUsage> {
            self.ordinary("process_rusage", usage(), 3)
        }
        fn thread_rusage(&mut self) -> Outcome<ResourceUsage> {
            self.ordinary("thread_rusage", usage(), 4)
        }
        fn statm(&mut self) -> Outcome<Statm> {
            self.ordinary(
                "statm",
                Statm {
                    size: 1,
                    resident: 2,
                    shared: 3,
                    text: 4,
                    lib: 5,
                    data: 6,
                    dt: 7,
                },
                5,
            )
        }
        fn status(&mut self) -> Outcome<StatusMemory> {
            self.ordinary(
                "status",
                StatusMemory {
                    resident_bytes: 1,
                    high_water_bytes: 2,
                },
                6,
            )
        }
        fn process_io(&mut self) -> Outcome<ProcessIo> {
            self.ordinary(
                "io",
                ProcessIo {
                    rchar: 1,
                    wchar: 2,
                    syscr: 3,
                    syscw: 4,
                    read_bytes: 5,
                    write_bytes: 6,
                    cancelled_write_bytes: 7,
                },
                7,
            )
        }
        fn file_length(&mut self, file: MeasuredFileIdentity<'_>) -> Outcome<FileLength> {
            assert_eq!(file.identity, "fd-7");
            self.ordinary(
                "file",
                FileLength {
                    bytes: 8,
                    source: FileLengthSource::Statx,
                    statx_only_fields: Ok(()),
                },
                8,
            )
        }
        fn open_perf(&mut self, event: PerfEvent) -> Outcome<PerfEvent> {
            self.calls.borrow_mut().push(format!("open:{event:?}"));
            match self.perf_fault.filter(|(e, _)| *e == event).map(|(_, f)| f) {
                Some(PerfFault::Unavailable) => {
                    Outcome::Unavailable(UnavailableReason::Unsupported)
                }
                Some(PerfFault::Permission) => Outcome::Permission(13),
                Some(PerfFault::Error) => Outcome::Error(ErrorReason::Errno(9)),
                Some(PerfFault::Overflow) => Outcome::Overflow(OverflowReason::PerfScaling),
                None => Outcome::Success(event),
            }
        }
        fn stop_perf(&mut self, _: &mut PerfEvent, event: PerfEvent) -> Outcome<PerfCounter> {
            let name = format!("stop:{event:?}");
            if self.mark(name) {
                Outcome::Overflow(OverflowReason::PerfScaling)
            } else {
                Outcome::Success(counter(event))
            }
        }
        fn cleanup_perf(&mut self, _: PerfEvent, event: PerfEvent) -> Outcome<()> {
            self.calls.borrow_mut().push(format!("cleanup:{event:?}"));
            if self.cleanup_fail.contains(&event) {
                Outcome::Error(ErrorReason::PerfCleanup(9))
            } else {
                Outcome::Success(())
            }
        }
        fn transition(
            &mut self,
            life: &mut Lifecycle,
            next: LifecycleState,
        ) -> Result<(), Failure> {
            self.calls.borrow_mut().push(format!("transition:{next:?}"));
            if self.transition_fail == Some(next) {
                self.transition_fail = None;
                Err(Failure::InvalidTransition {
                    from: life.state(),
                    to: next,
                })
            } else {
                life.transition(next)
            }
        }
    }
    struct Action {
        calls: usize,
        fail: bool,
        ledger: Rc<RefCell<Vec<String>>>,
    }
    impl Action {
        fn new(calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                calls: 0,
                fail: false,
                ledger: calls,
            }
        }
    }
    impl MeasuredAction for Action {
        fn invoke(&mut self) -> Result<(), String> {
            self.calls += 1;
            self.ledger.borrow_mut().push("action".into());
            if self.fail {
                Err("action failed".into())
            } else {
                Ok(())
            }
        }
    }
    fn run(b: &mut Synthetic) -> (ObservationOutcome, usize) {
        let mut a = Action::new(b.calls.clone());
        let out = observe(
            &plan(),
            MeasuredFileIdentity { identity: "fd-7" },
            b,
            &mut a,
        );
        (out, a.calls)
    }
    fn invalid(out: ObservationOutcome) -> InvalidObservation {
        match out {
            ObservationOutcome::Invalid(v) => v,
            _ => panic!("must be invalid"),
        }
    }

    #[test]
    fn success_contract_has_shared_action_order_identity_units_and_cleanup() {
        let mut b = Synthetic::default();
        let (out, calls) = run(&mut b);
        let ObservationOutcome::Complete(done) = out else {
            panic!()
        };
        assert_eq!(calls, 1);
        assert_eq!(done.cleanup, CleanupStatus::Successful);
        let log = b.calls.borrow();
        let start = log.iter().position(|v| v == "monotonic").unwrap();
        let action = log.iter().position(|v| v == "action").unwrap();
        let end = log.iter().rposition(|v| v == "monotonic").unwrap();
        assert!(start < action && action < end);
        assert_eq!(done.observation.elapsed_ns, Some(5));
        assert_eq!(done.observation.process_rusage.scope, SourceScope::Process);
        assert_eq!(
            done.observation.thread_rusage.scope,
            SourceScope::MeasuredThread
        );
        assert_eq!(done.observation.file_length.unit, Unit::Bytes);
        for (entry, event) in done.observation.perf.iter().zip(SourceList::R31.perf) {
            assert_eq!(entry.event, event);
            assert_eq!(entry.scope, SourceScope::MeasuredThread);
        }
        assert_eq!(done.ledger.last(), Some(&LifecycleState::Complete));
        for event in SourceList::R31.perf {
            assert_eq!(
                log.iter()
                    .filter(|v| **v == format!("cleanup:{event:?}"))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn lifecycle_exhaustively_rejects_every_illegal_duplicate_and_terminal_transition() {
        use LifecycleState::*;
        let states = [
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
        ];
        for from in states {
            for to in states {
                let mut life = Lifecycle {
                    state: from,
                    ledger: vec![from],
                };
                let legal = matches!(
                    (from, to),
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
                assert_eq!(life.transition(to).is_ok(), legal, "{from:?} -> {to:?}");
            }
        }
    }

    #[test]
    fn every_non_perf_before_and_after_and_both_monotonic_fail_closed() {
        for call in [
            "realtime",
            "process_rusage",
            "thread_rusage",
            "statm",
            "status",
            "io",
            "file",
        ] {
            for occurrence in 1..=2 {
                let mut b = Synthetic {
                    fail_call: Some(call.into()),
                    fail_occurrence: occurrence,
                    ..Default::default()
                };
                let (out, _) = run(&mut b);
                assert!(
                    matches!(out, ObservationOutcome::Invalid(_)),
                    "{call} #{occurrence}"
                );
            }
        }
        for occurrence in 1..=2 {
            let mut b = Synthetic {
                fail_call: Some("monotonic".into()),
                fail_occurrence: occurrence,
                ..Default::default()
            };
            let (out, _) = run(&mut b);
            assert!(matches!(out, ObservationOutcome::Invalid(_)));
        }
    }

    #[test]
    fn elapsed_reversal_and_checked_overflow_are_invalid() {
        for clocks in [[5, 4], [i128::MIN, i128::MAX]] {
            let mut b = Synthetic {
                next_clock: clocks.into(),
                ..Default::default()
            };
            let (out, _) = run(&mut b);
            let v = invalid(out);
            assert!(matches!(
                v.primary_failure,
                Failure::MonotonicReversal { .. } | Failure::ElapsedOverflow
            ));
        }
    }

    #[test]
    fn every_perf_event_retains_each_open_outcome_and_post_open_failure() {
        for event in SourceList::R31.perf {
            for fault in [
                PerfFault::Unavailable,
                PerfFault::Permission,
                PerfFault::Error,
                PerfFault::Overflow,
            ] {
                let mut b = Synthetic {
                    perf_fault: Some((event, fault)),
                    ..Default::default()
                };
                let (out, _) = run(&mut b);
                let entry = match &out {
                    ObservationOutcome::Complete(v) => &v.observation.perf,
                    ObservationOutcome::Invalid(v) => &v.observation.perf,
                }
                .iter()
                .find(|v| v.event == event)
                .unwrap();
                assert!(entry.outcome.is_some());
                assert_eq!(
                    matches!(out, ObservationOutcome::Complete(_)),
                    matches!(fault, PerfFault::Unavailable | PerfFault::Permission)
                );
            }
            let mut b = Synthetic {
                fail_call: Some(format!("stop:{event:?}")),
                ..Default::default()
            };
            let (out, _) = run(&mut b);
            assert!(matches!(invalid(out).primary_failure,
                Failure::Source { phase: Phase::PerfStop(e), .. } if e == event));
        }
    }

    #[test]
    fn reverse_cleanup_after_every_open_failure_transition_failure_and_action_unwind() {
        for (index, event) in SourceList::R31.perf.into_iter().enumerate() {
            let mut b = Synthetic {
                perf_fault: Some((event, PerfFault::Error)),
                ..Default::default()
            };
            let _ = run(&mut b);
            let expected: Vec<_> = SourceList::R31.perf[..index]
                .iter()
                .rev()
                .map(|e| format!("cleanup:{e:?}"))
                .collect();
            let actual: Vec<_> = b
                .calls
                .borrow()
                .iter()
                .filter(|v| v.starts_with("cleanup:"))
                .cloned()
                .collect();
            assert_eq!(actual, expected);
        }
        for state in [
            LifecycleState::CountersArmed,
            LifecycleState::Measuring,
            LifecycleState::ActionCompleted,
            LifecycleState::CountersStopped,
            LifecycleState::AfterCaptured,
            LifecycleState::Cleaned,
            LifecycleState::Complete,
        ] {
            let mut b = Synthetic {
                transition_fail: Some(state),
                ..Default::default()
            };
            let (out, _) = run(&mut b);
            assert!(matches!(out, ObservationOutcome::Invalid(_)));
            assert_eq!(
                b.calls
                    .borrow()
                    .iter()
                    .filter(|v| v.starts_with("cleanup:"))
                    .count(),
                4
            );
        }
        let mut b = Synthetic::default();
        let mut a = Action::new(b.calls.clone());
        a.fail = true;
        assert!(matches!(
            observe(
                &plan(),
                MeasuredFileIdentity { identity: "fd-7" },
                &mut b,
                &mut a
            ),
            ObservationOutcome::Invalid(_)
        ));
        assert_eq!(
            b.calls
                .borrow()
                .iter()
                .filter(|v| v.starts_with("cleanup:"))
                .count(),
            4
        );
    }

    #[test]
    fn cleanup_only_primary_multiple_failures_order_and_no_double_cleanup() {
        let mut b = Synthetic {
            cleanup_fail: vec![
                PerfEvent::ContextSwitches,
                PerfEvent::PageFaults,
                PerfEvent::Instructions,
            ],
            ..Default::default()
        };
        let (out, _) = run(&mut b);
        let v = invalid(out);
        assert!(matches!(
            v.primary_failure,
            Failure::Source {
                phase: Phase::Cleanup(PerfEvent::ContextSwitches),
                ..
            }
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
                Phase::Cleanup(PerfEvent::PageFaults),
                Phase::Cleanup(PerfEvent::Instructions)
            ]
        );
        for event in SourceList::R31.perf {
            assert_eq!(
                b.calls
                    .borrow()
                    .iter()
                    .filter(|v| **v == format!("cleanup:{event:?}"))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn tracefs_reasons_and_evidence_are_exact() {
        assert!(validate(&plan()).is_ok());
        let mut supported = plan();
        supported.tracefs = traces(TraceMissingState::Unsupported);
        assert!(validate(&supported).is_ok());
        supported.tracefs[1].preflight_evidence = None;
        assert!(validate(&supported).is_err());
        let mut wrong = plan();
        wrong.tracefs[0].reason = "similar but unauthorized";
        assert!(validate(&wrong).is_err());
        let mut evidence = plan();
        evidence.tracefs[0].preflight_evidence = Some("forbidden".into());
        assert!(validate(&evidence).is_err());
    }
}
