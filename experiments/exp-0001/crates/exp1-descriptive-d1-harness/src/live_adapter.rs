//! R32's live-call-capable, but deliberately uncalled, internal adapter.

use crate::linux_capture::{
    self, Clock, ErrorReason, FileLength, Outcome, PerfCleanupState, PerfCounter, PerfEvent,
    PerfEventSession, ProcessIo, ResourceScope, ResourceUsage, Statm, StatusMemory,
};
use crate::orchestration::{CaptureBoundary, MeasuredFileReference};

trait LiveOperations {
    type PerfOwner;
    fn clock(&mut self, clock: Clock) -> Outcome<i128>;
    fn rusage(&mut self, scope: ResourceScope) -> Outcome<ResourceUsage>;
    fn statm(&mut self) -> Outcome<Statm>;
    fn status(&mut self) -> Outcome<StatusMemory>;
    fn process_io(&mut self) -> Outcome<ProcessIo>;
    fn file_length(&mut self, file: MeasuredFileReference<'_>) -> Outcome<FileLength>;
    fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner>;
    fn stop_perf(&mut self, owner: &mut Self::PerfOwner) -> Outcome<PerfCounter>;
    fn cleanup_perf(&mut self, owner: Self::PerfOwner) -> Outcome<()>;
}

struct SystemOperations<'a> {
    cleanup: &'a PerfCleanupState,
}

impl<'a> LiveOperations for SystemOperations<'a> {
    type PerfOwner = PerfEventSession<'a>;

    fn clock(&mut self, clock: Clock) -> Outcome<i128> {
        linux_capture::clock_time(clock)
    }
    fn rusage(&mut self, scope: ResourceScope) -> Outcome<ResourceUsage> {
        linux_capture::resource_usage(scope)
    }
    fn statm(&mut self) -> Outcome<Statm> {
        linux_capture::read_statm()
    }
    fn status(&mut self) -> Outcome<StatusMemory> {
        linux_capture::read_status()
    }
    fn process_io(&mut self) -> Outcome<ProcessIo> {
        linux_capture::read_io()
    }
    fn file_length(&mut self, file: MeasuredFileReference<'_>) -> Outcome<FileLength> {
        match file.file() {
            Some(capability) => linux_capture::open_file_length(capability),
            None => Outcome::Error(ErrorReason::MissingFileCapability),
        }
    }
    fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner> {
        PerfEventSession::open(event, self.cleanup)
    }
    fn stop_perf(&mut self, owner: &mut Self::PerfOwner) -> Outcome<PerfCounter> {
        owner.stop()
    }
    fn cleanup_perf(&mut self, owner: Self::PerfOwner) -> Outcome<()> {
        owner.cleanup()
    }
}

struct Adapter<O> {
    operations: O,
}

impl<O: LiveOperations> CaptureBoundary for Adapter<O> {
    type PerfOwner = O::PerfOwner;

    fn realtime(&mut self) -> Outcome<i128> {
        self.operations.clock(Clock::Realtime)
    }
    fn monotonic_raw(&mut self) -> Outcome<i128> {
        self.operations.clock(Clock::MonotonicRaw)
    }
    fn process_rusage(&mut self) -> Outcome<ResourceUsage> {
        self.operations.rusage(ResourceScope::Process)
    }
    fn thread_rusage(&mut self) -> Outcome<ResourceUsage> {
        self.operations.rusage(ResourceScope::Thread)
    }
    fn statm(&mut self) -> Outcome<Statm> {
        self.operations.statm()
    }
    fn status(&mut self) -> Outcome<StatusMemory> {
        self.operations.status()
    }
    fn process_io(&mut self) -> Outcome<ProcessIo> {
        self.operations.process_io()
    }
    fn file_length(&mut self, file: MeasuredFileReference<'_>) -> Outcome<FileLength> {
        self.operations.file_length(file)
    }
    fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner> {
        self.operations.open_perf(event)
    }
    fn stop_perf(&mut self, owner: &mut Self::PerfOwner, _: PerfEvent) -> Outcome<PerfCounter> {
        self.operations.stop_perf(owner)
    }
    fn cleanup_perf(&mut self, owner: Self::PerfOwner, _: PerfEvent) -> Outcome<()> {
        self.operations.cleanup_perf(owner)
    }
}

/// The sole R32 Linux/x86_64 bridge to the existing live wrappers.
///
/// Construction performs no probing or host call.  No caller is provided by
/// this crate; possessing this value does not validate a target.
pub struct LiveCaptureBoundary<'a> {
    inner: Adapter<SystemOperations<'a>>,
}

impl<'a> LiveCaptureBoundary<'a> {
    pub fn new(cleanup: &'a PerfCleanupState) -> Self {
        Self {
            inner: Adapter {
                operations: SystemOperations { cleanup },
            },
        }
    }
}

impl<'a> CaptureBoundary for LiveCaptureBoundary<'a> {
    type PerfOwner = PerfEventSession<'a>;

    fn realtime(&mut self) -> Outcome<i128> {
        self.inner.realtime()
    }
    fn monotonic_raw(&mut self) -> Outcome<i128> {
        self.inner.monotonic_raw()
    }
    fn process_rusage(&mut self) -> Outcome<ResourceUsage> {
        self.inner.process_rusage()
    }
    fn thread_rusage(&mut self) -> Outcome<ResourceUsage> {
        self.inner.thread_rusage()
    }
    fn statm(&mut self) -> Outcome<Statm> {
        self.inner.statm()
    }
    fn status(&mut self) -> Outcome<StatusMemory> {
        self.inner.status()
    }
    fn process_io(&mut self) -> Outcome<ProcessIo> {
        self.inner.process_io()
    }
    fn file_length(&mut self, file: MeasuredFileReference<'_>) -> Outcome<FileLength> {
        self.inner.file_length(file)
    }
    fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner> {
        self.inner.open_perf(event)
    }
    fn stop_perf(&mut self, owner: &mut Self::PerfOwner, event: PerfEvent) -> Outcome<PerfCounter> {
        self.inner.stop_perf(owner, event)
    }
    fn cleanup_perf(&mut self, owner: Self::PerfOwner, event: PerfEvent) -> Outcome<()> {
        self.inner.cleanup_perf(owner, event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linux_capture::{FileLengthSource, UnavailableReason};
    use std::cell::Cell;
    use std::os::fd::{AsRawFd, RawFd};

    struct NeverReadFd<'a>(&'a Cell<usize>);
    impl AsRawFd for NeverReadFd<'_> {
        fn as_raw_fd(&self) -> RawFd {
            self.0.set(self.0.get() + 1);
            77
        }
    }

    #[derive(Default)]
    struct FakeOperations {
        calls: Vec<String>,
    }

    impl LiveOperations for FakeOperations {
        type PerfOwner = PerfEvent;
        fn clock(&mut self, clock: Clock) -> Outcome<i128> {
            self.calls.push(format!("clock:{clock:?}"));
            Outcome::Success(1)
        }
        fn rusage(&mut self, scope: ResourceScope) -> Outcome<ResourceUsage> {
            self.calls.push(format!("rusage:{scope:?}"));
            Outcome::Error(ErrorReason::Errno(10))
        }
        fn statm(&mut self) -> Outcome<Statm> {
            self.calls.push("statm".into());
            Outcome::Unavailable(UnavailableReason::NotFound)
        }
        fn status(&mut self) -> Outcome<StatusMemory> {
            self.calls.push("status".into());
            Outcome::Permission(13)
        }
        fn process_io(&mut self) -> Outcome<ProcessIo> {
            self.calls.push("io".into());
            Outcome::Error(ErrorReason::Io(std::io::ErrorKind::Other))
        }
        fn file_length(&mut self, file: MeasuredFileReference<'_>) -> Outcome<FileLength> {
            self.calls.push(format!(
                "file:{}:{}",
                file.identity(),
                file.has_file_capability()
            ));
            Outcome::Success(FileLength {
                bytes: 9,
                source: FileLengthSource::Statx,
                statx_only_fields: Ok(()),
            })
        }
        fn open_perf(&mut self, event: PerfEvent) -> Outcome<Self::PerfOwner> {
            self.calls.push(format!("open:{event:?}"));
            Outcome::Success(event)
        }
        fn stop_perf(&mut self, owner: &mut Self::PerfOwner) -> Outcome<PerfCounter> {
            self.calls.push(format!("stop:{owner:?}"));
            Outcome::Unavailable(UnavailableReason::Unsupported)
        }
        fn cleanup_perf(&mut self, owner: Self::PerfOwner) -> Outcome<()> {
            self.calls.push(format!("cleanup:{owner:?}"));
            Outcome::Success(())
        }
    }

    #[test]
    fn injected_adapter_maps_every_operation_scope_event_and_file_representation() {
        let reads = Cell::new(0);
        let descriptor = NeverReadFd(&reads);
        let mut boundary = Adapter {
            operations: FakeOperations::default(),
        };
        assert!(matches!(boundary.realtime(), Outcome::Success(1)));
        assert!(matches!(boundary.monotonic_raw(), Outcome::Success(1)));
        assert!(matches!(boundary.process_rusage(), Outcome::Error(_)));
        assert!(matches!(boundary.thread_rusage(), Outcome::Error(_)));
        assert!(matches!(boundary.statm(), Outcome::Unavailable(_)));
        assert!(matches!(boundary.status(), Outcome::Permission(13)));
        assert!(matches!(boundary.process_io(), Outcome::Error(_)));
        assert!(matches!(
            boundary.file_length(MeasuredFileReference::identity_only("stable")),
            Outcome::Success(_)
        ));
        assert!(matches!(
            boundary.file_length(MeasuredFileReference::borrowed("stable", &descriptor)),
            Outcome::Success(_)
        ));
        for event in [
            PerfEvent::CpuCycles,
            PerfEvent::Instructions,
            PerfEvent::PageFaults,
            PerfEvent::ContextSwitches,
        ] {
            let Outcome::Success(mut owner) = boundary.open_perf(event) else {
                panic!()
            };
            assert!(matches!(
                boundary.stop_perf(&mut owner, event),
                Outcome::Unavailable(_)
            ));
            assert!(matches!(
                boundary.cleanup_perf(owner, event),
                Outcome::Success(())
            ));
        }
        assert_eq!(reads.get(), 0, "the injected adapter never observes the fd");
        assert_eq!(
            boundary.operations.calls,
            [
                "clock:Realtime",
                "clock:MonotonicRaw",
                "rusage:Process",
                "rusage:Thread",
                "statm",
                "status",
                "io",
                "file:stable:false",
                "file:stable:true",
                "open:CpuCycles",
                "stop:CpuCycles",
                "cleanup:CpuCycles",
                "open:Instructions",
                "stop:Instructions",
                "cleanup:Instructions",
                "open:PageFaults",
                "stop:PageFaults",
                "cleanup:PageFaults",
                "open:ContextSwitches",
                "stop:ContextSwitches",
                "cleanup:ContextSwitches",
            ]
        );
    }

    #[test]
    fn live_boundary_construction_is_inert_and_identity_only_file_fails_closed() {
        let cleanup = PerfCleanupState::default();
        let mut boundary = LiveCaptureBoundary::new(&cleanup);
        assert_eq!(
            boundary.file_length(MeasuredFileReference::identity_only("retained-id")),
            Outcome::Error(ErrorReason::MissingFileCapability)
        );
        assert!(!cleanup.cleanup_failed());
    }
}
