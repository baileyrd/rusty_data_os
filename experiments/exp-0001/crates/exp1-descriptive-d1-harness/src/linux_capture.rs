//! Exact, dependency-free R29 Linux/x86_64 ABI boundary.

use std::fs;
use std::io;
use std::os::fd::{AsRawFd, RawFd};

pub const CLOCK_REALTIME: i32 = 0;
pub const CLOCK_MONOTONIC_RAW: i32 = 4;
pub const RUSAGE_SELF: i32 = 0;
pub const RUSAGE_THREAD: i32 = 1;
pub const AT_EMPTY_PATH: i32 = 0x1000;
pub const AT_STATX_SYNC_AS_STAT: i32 = 0x0000;
pub const STATX_SIZE: u32 = 0x0000_0200;

pub const EPERM: i32 = 1;
pub const ENOENT: i32 = 2;
pub const EBADF: i32 = 9;
pub const EACCES: i32 = 13;
pub const ENODEV: i32 = 19;
pub const ENOTDIR: i32 = 20;
pub const EINVAL: i32 = 22;
pub const ENOSYS: i32 = 38;
pub const EOVERFLOW: i32 = 75;
pub const ENXIO: i32 = 6;
pub const ENOMEM: i32 = 12;
pub const EBUSY: i32 = 16;
pub const ENFILE: i32 = 23;
pub const EMFILE: i32 = 24;
pub const EOPNOTSUPP: i32 = 95;

pub const PERF_EVENT_OPEN_SYSCALL: i64 = 298;
pub const PERF_FLAG_FD_CLOEXEC: u64 = 0x8;
pub const PERF_ATTR_SIZE_VER0: u32 = 64;
pub const PERF_FORMAT_TOTAL_TIME_ENABLED: u64 = 0x1;
pub const PERF_FORMAT_TOTAL_TIME_RUNNING: u64 = 0x2;
pub const PERF_READ_FORMAT: u64 = 0x3;
pub const PERF_EVENT_IOC_ENABLE: u64 = 0x2400;
pub const PERF_EVENT_IOC_DISABLE: u64 = 0x2401;
pub const PERF_EVENT_IOC_RESET: u64 = 0x2403;
const PERF_ATTR_DISABLED: u64 = 1;
const PERF_READ_BYTES: usize = 24;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome<T> {
    Success(T),
    Unavailable(UnavailableReason),
    Permission(i32),
    Overflow(OverflowReason),
    Error(ErrorReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnavailableReason {
    Interface(i32),
    MissingStatxSize,
    NotFound,
    Unsupported,
    StatxOnlyAfterFstat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverflowReason {
    Arithmetic,
    FileSize,
    NumericField,
    PerfScaling,
    PerfErrno(i32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorReason {
    Errno(i32),
    InvalidFraction,
    NegativeCounter,
    NegativeFileSize,
    Io(io::ErrorKind),
    InvalidUtf8,
    Parse(ParseReason),
    PerfShortRead(isize),
    PerfInvalidTime,
    PerfDecrease,
    PerfLifecycle,
    PerfCleanup(i32),
    PerfUnexpectedReturn(i32),
    PerfCleanupUnexpected(i32),
    PerfEventMismatch {
        expected: PerfEvent,
        actual: PerfEvent,
    },
    MissingFileCapability,
}

/// Source identity for an R30 counter. These values are intentionally distinct
/// from the fields in [`ResourceUsage`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerfEvent {
    CpuCycles,
    Instructions,
    PageFaults,
    ContextSwitches,
}

impl PerfEvent {
    const ALL: [Self; 4] = [
        Self::CpuCycles,
        Self::Instructions,
        Self::PageFaults,
        Self::ContextSwitches,
    ];

    const fn selector(self) -> (u32, u64) {
        match self {
            Self::CpuCycles => (0, 0),
            Self::Instructions => (0, 1),
            Self::PageFaults => (1, 2),
            Self::ContextSwitches => (1, 3),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerfCounter {
    pub event: PerfEvent,
    pub raw_count: u64,
    pub time_enabled_ns: u64,
    pub time_running_ns: u64,
    pub multiplexed: bool,
    pub scaled_count: Outcome<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseReason {
    NonAscii,
    LineCount,
    TokenCount,
    MalformedLine,
    MissingField,
    DuplicateField,
    SignedValue,
    InvalidNumber,
    InvalidUnit,
    TrailingToken,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Clock {
    Realtime,
    MonotonicRaw,
}

impl Clock {
    const fn id(self) -> i32 {
        match self {
            Self::Realtime => CLOCK_REALTIME,
            Self::MonotonicRaw => CLOCK_MONOTONIC_RAW,
        }
    }
}

impl TryFrom<i32> for Clock {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            CLOCK_REALTIME => Ok(Self::Realtime),
            CLOCK_MONOTONIC_RAW => Ok(Self::MonotonicRaw),
            other => Err(other),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceScope {
    Process,
    Thread,
}

impl ResourceScope {
    const fn id(self) -> i32 {
        match self {
            Self::Process => RUSAGE_SELF,
            Self::Thread => RUSAGE_THREAD,
        }
    }
}

impl TryFrom<i32> for ResourceScope {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            RUSAGE_SELF => Ok(Self::Process),
            RUSAGE_THREAD => Ok(Self::Thread),
            other => Err(other),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceUsage {
    pub user_nanoseconds: i128,
    pub system_nanoseconds: i128,
    pub maximum_resident_bytes: u64,
    pub minor_faults: u64,
    pub major_faults: u64,
    pub input_blocks: u64,
    pub output_blocks: u64,
    pub voluntary_context_switches: u64,
    pub involuntary_context_switches: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileLengthSource {
    Statx,
    FstatFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileLength {
    pub bytes: i64,
    pub source: FileLengthSource,
    pub statx_only_fields: Result<(), UnavailableReason>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Statm {
    pub size: u64,
    pub resident: u64,
    pub shared: u64,
    pub text: u64,
    pub lib: u64,
    pub data: u64,
    pub dt: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatusMemory {
    pub resident_bytes: u64,
    pub high_water_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessIo {
    pub rchar: u64,
    pub wchar: u64,
    pub syscr: u64,
    pub syscw: u64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub cancelled_write_bytes: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rusage {
    ru_utime: Timeval,
    ru_stime: Timeval,
    ru_maxrss: i64,
    _pad_40: [u8; 24],
    ru_minflt: i64,
    ru_majflt: i64,
    _pad_80: [u8; 8],
    ru_inblock: i64,
    ru_oublock: i64,
    _pad_104: [u8; 24],
    ru_nvcsw: i64,
    ru_nivcsw: i64,
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct Statx {
    stx_mask: u32,
    _pad_4: [u8; 36],
    stx_size: u64,
    _pad_48: [u8; 208],
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct Stat {
    _pad_0: [u8; 48],
    st_size: i64,
    _pad_56: [u8; 88],
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PerfEventAttr {
    event_type: u32,
    size: u32,
    config: u64,
    sample_period_or_freq: u64,
    sample_type: u64,
    read_format: u64,
    flags: u64,
    wakeup_events_or_watermark: u32,
    bp_type: u32,
    config1_or_bp_addr: u64,
}

impl PerfEventAttr {
    const fn for_event(event: PerfEvent) -> Self {
        let (event_type, config) = event.selector();
        Self {
            event_type,
            size: PERF_ATTR_SIZE_VER0,
            config,
            sample_period_or_freq: 0,
            sample_type: 0,
            read_format: PERF_READ_FORMAT,
            flags: PERF_ATTR_DISABLED,
            wakeup_events_or_watermark: 0,
            bp_type: 0,
            config1_or_bp_addr: 0,
        }
    }
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PerfSnapshot {
    pub raw_count: u64,
    pub time_enabled_ns: u64,
    pub time_running_ns: u64,
}

const _: () = {
    assert!(std::mem::size_of::<PerfEventAttr>() == 64);
    assert!(std::mem::align_of::<PerfEventAttr>() == 8);
    assert!(std::mem::offset_of!(PerfEventAttr, event_type) == 0);
    assert!(std::mem::offset_of!(PerfEventAttr, size) == 4);
    assert!(std::mem::offset_of!(PerfEventAttr, config) == 8);
    assert!(std::mem::offset_of!(PerfEventAttr, sample_period_or_freq) == 16);
    assert!(std::mem::offset_of!(PerfEventAttr, sample_type) == 24);
    assert!(std::mem::offset_of!(PerfEventAttr, read_format) == 32);
    assert!(std::mem::offset_of!(PerfEventAttr, flags) == 40);
    assert!(std::mem::offset_of!(PerfEventAttr, wakeup_events_or_watermark) == 48);
    assert!(std::mem::offset_of!(PerfEventAttr, bp_type) == 52);
    assert!(std::mem::offset_of!(PerfEventAttr, config1_or_bp_addr) == 56);
    assert!(std::mem::size_of::<PerfSnapshot>() == PERF_READ_BYTES);
    assert!(std::mem::align_of::<PerfSnapshot>() == 8);
    assert!(std::mem::offset_of!(PerfSnapshot, raw_count) == 0);
    assert!(std::mem::offset_of!(PerfSnapshot, time_enabled_ns) == 8);
    assert!(std::mem::offset_of!(PerfSnapshot, time_running_ns) == 16);
    assert!(PERF_ATTR_DISABLED == 1);
    assert!(PERF_READ_FORMAT == 3);
};

unsafe extern "C" {
    fn clock_gettime(clock_id: i32, result: *mut Timespec) -> i32;
    fn clock_getres(clock_id: i32, result: *mut Timespec) -> i32;
    fn getrusage(who: i32, result: *mut Rusage) -> i32;
    fn statx(dirfd: i32, pathname: *const u8, flags: i32, mask: u32, result: *mut Statx) -> i32;
    fn fstat(fd: i32, result: *mut Stat) -> i32;
    fn syscall(number: i64, ...) -> i64;
    fn ioctl(fd: i32, request: u64, ...) -> i32;
    fn read(fd: i32, result: *mut u8, count: usize) -> isize;
    fn close(fd: i32) -> i32;
}

trait PerfBoundary: Copy {
    fn open(&self, attr: &PerfEventAttr) -> Result<RawFd, i32>;
    fn ioctl(&self, fd: RawFd, request: u64) -> Result<(), BoundaryCallError>;
    fn read(&self, fd: RawFd, result: &mut PerfSnapshot) -> Result<usize, BoundaryReadError>;
    fn close(&self, fd: RawFd) -> Result<(), BoundaryCallError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundaryCallError {
    Errno(i32),
    Unexpected(i32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundaryReadError {
    Errno(i32),
    Unexpected(isize),
}

#[derive(Clone, Copy)]
struct GlibcPerfBoundary;

impl PerfBoundary for GlibcPerfBoundary {
    fn open(&self, attr: &PerfEventAttr) -> Result<RawFd, i32> {
        // SAFETY: The frozen x86_64 glibc variadic ABI promotes every integer
        // argument to signed 64 bits and `attr` points to the exact 64-byte V0 layout.
        let result = unsafe {
            syscall(
                PERF_EVENT_OPEN_SYSCALL,
                attr as *const PerfEventAttr,
                0_i64,
                -1_i64,
                -1_i64,
                PERF_FLAG_FD_CLOEXEC as i64,
            )
        };
        if result >= 0 {
            i32::try_from(result).map_err(|_| EOVERFLOW)
        } else {
            Err(last_errno())
        }
    }

    fn ioctl(&self, fd: RawFd, request: u64) -> Result<(), BoundaryCallError> {
        // SAFETY: The request is one of the frozen no-argument perf ioctls.
        let result = unsafe { ioctl(fd, request, 0_i64) };
        classify_zero_result(result, last_errno)
    }

    fn read(&self, fd: RawFd, result: &mut PerfSnapshot) -> Result<usize, BoundaryReadError> {
        // SAFETY: `result` provides exactly 24 writable bytes for the frozen read format.
        let count = unsafe { read(fd, (result as *mut PerfSnapshot).cast(), PERF_READ_BYTES) };
        if count < 0 {
            Err(BoundaryReadError::Errno(last_errno()))
        } else if count as usize == PERF_READ_BYTES {
            Ok(PERF_READ_BYTES)
        } else {
            Err(BoundaryReadError::Unexpected(count))
        }
    }

    fn close(&self, fd: RawFd) -> Result<(), BoundaryCallError> {
        // SAFETY: The uniquely owned descriptor is closed exactly once.
        let result = unsafe { close(fd) };
        classify_zero_result(result, last_errno)
    }
}

fn classify_zero_result(result: i32, errno: impl FnOnce() -> i32) -> Result<(), BoundaryCallError> {
    match result {
        0 => Ok(()),
        -1 => Err(BoundaryCallError::Errno(errno())),
        unexpected => Err(BoundaryCallError::Unexpected(unexpected)),
    }
}

#[derive(Clone, Debug, Default)]
pub struct PerfCleanupState {
    failure: std::rc::Rc<std::cell::Cell<Option<BoundaryCallError>>>,
}

impl PerfCleanupState {
    pub fn cleanup_failed(&self) -> bool {
        self.failure.get().is_some()
    }

    pub fn cleanup_errno(&self) -> Option<i32> {
        match self.failure.get() {
            Some(BoundaryCallError::Errno(errno)) => Some(errno),
            _ => None,
        }
    }

    pub fn cleanup_unexpected_return(&self) -> Option<i32> {
        match self.failure.get() {
            Some(BoundaryCallError::Unexpected(result)) => Some(result),
            _ => None,
        }
    }
}

struct OwnedPerfFd<B: PerfBoundary> {
    fd: Option<RawFd>,
    boundary: B,
    cleanup: PerfCleanupState,
    event: PerfEvent,
}

impl<B: PerfBoundary> OwnedPerfFd<B> {
    fn finalize(mut self) -> Result<(), BoundaryCallError> {
        let fd = self.fd.take().expect("owned descriptor is present");
        let result = self.boundary.close(fd);
        if result.is_err() {
            self.cleanup.failure.set(result.err());
        }
        result
    }
}

impl<B: PerfBoundary> Drop for OwnedPerfFd<B> {
    fn drop(&mut self) {
        if let Some(fd) = self.fd.take()
            && let Err(failure) = self.boundary.close(fd)
        {
            self.cleanup.failure.set(Some(failure));
        }
    }
}

/// A disabled-at-open, independently owned set of the four R30 perf counters.
/// Construction makes live calls; R30 deliberately authorizes no caller or
/// measured interval yet.
pub struct PerfCounterSession<'a> {
    boundary: GlibcPerfBoundary,
    owners: PerfFdOwners<'a, GlibcPerfBoundary>,
    lifetime: std::marker::PhantomData<&'a PerfCleanupState>,
}

/// One independently owned R30 counter.  This is the ownership unit used by
/// the R32 adapter; it neither groups nor suppresses outcomes from other
/// events.
pub struct PerfEventSession<'a> {
    owner: Option<OwnedPerfFd<GlibcPerfBoundary>>,
    stopped: bool,
    lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a> PerfEventSession<'a> {
    /// Opens, resets, and enables exactly one event.
    pub fn open(event: PerfEvent) -> Outcome<Self> {
        match open_one_perf(GlibcPerfBoundary, PerfCleanupState::default(), event) {
            Outcome::Success(owner) => Outcome::Success(Self {
                owner: Some(owner),
                stopped: false,
                lifetime: std::marker::PhantomData,
            }),
            outcome => outcome.map_type(),
        }
    }

    pub(crate) fn event(&self) -> PerfEvent {
        self.owner
            .as_ref()
            .expect("owned descriptor is present")
            .event
    }

    /// Disables and reads this event.  Closing remains a separate operation so
    /// the orchestrator can retain stop and cleanup failures independently.
    pub fn stop(&mut self) -> Outcome<PerfCounter> {
        stop_event_session(
            GlibcPerfBoundary,
            self.owner.as_ref().expect("owned descriptor is present"),
            &mut self.stopped,
        )
    }

    /// Closes the uniquely owned descriptor exactly once.
    pub fn cleanup(mut self) -> Outcome<()> {
        let owner = self.owner.take().expect("owned descriptor is present");
        match owner.finalize() {
            Ok(()) => Outcome::Success(()),
            Err(error) => classify_perf_close_error(error),
        }
    }
}

fn stop_event_session<B: PerfBoundary>(
    boundary: B,
    owner: &OwnedPerfFd<B>,
    stopped: &mut bool,
) -> Outcome<PerfCounter> {
    if *stopped {
        return Outcome::Error(ErrorReason::PerfLifecycle);
    }
    *stopped = true;
    stop_one_perf(boundary, owner)
}

struct PerfFdOwners<'a, B: PerfBoundary> {
    fds: Vec<OwnedPerfFd<B>>,
    lifetime: std::marker::PhantomData<&'a PerfCleanupState>,
}

impl<B: PerfBoundary> PerfFdOwners<'_, B> {
    fn close_reverse(&mut self) {
        drop_owners_reverse(&mut self.fds);
    }
}

impl<B: PerfBoundary> Drop for PerfFdOwners<'_, B> {
    fn drop(&mut self) {
        self.close_reverse();
    }
}

impl Drop for PerfCounterSession<'_> {
    fn drop(&mut self) {
        self.owners.close_reverse();
    }
}

impl<'a> PerfCounterSession<'a> {
    pub fn open(cleanup: &'a PerfCleanupState) -> Outcome<Self> {
        let boundary = GlibcPerfBoundary;
        match open_perf_session(boundary, cleanup) {
            Outcome::Success(fds) => Outcome::Success(Self {
                boundary,
                owners: PerfFdOwners {
                    fds,
                    lifetime: std::marker::PhantomData,
                },
                lifetime: std::marker::PhantomData,
            }),
            outcome => outcome.map_type(),
        }
    }

    pub fn finalize(mut self) -> Outcome<[PerfCounter; 4]> {
        let result = finish_perf_session(self.boundary, &mut self.owners.fds);
        self.owners.fds.clear();
        result
    }
}

fn open_perf_session<B: PerfBoundary>(
    boundary: B,
    cleanup: &PerfCleanupState,
) -> Outcome<Vec<OwnedPerfFd<B>>> {
    let mut owners = Vec::with_capacity(4);
    for event in PerfEvent::ALL {
        let fd = match boundary.open(&PerfEventAttr::for_event(event)) {
            Ok(fd) => fd,
            Err(errno) => {
                drop_owners_reverse(&mut owners);
                return if cleanup.cleanup_failed() {
                    classify_sticky_cleanup(cleanup)
                } else {
                    classify_perf_open_errno(errno)
                };
            }
        };
        owners.push(OwnedPerfFd {
            fd: Some(fd),
            boundary,
            cleanup: cleanup.clone(),
            event,
        });
        for request in [PERF_EVENT_IOC_RESET, PERF_EVENT_IOC_ENABLE] {
            if let Err(error) = boundary.ioctl(fd, request) {
                drop_owners_reverse(&mut owners);
                return if cleanup.cleanup_failed() {
                    classify_sticky_cleanup(cleanup)
                } else {
                    classify_perf_boundary_error(error)
                };
            }
        }
    }
    Outcome::Success(owners)
}

fn open_one_perf<B: PerfBoundary>(
    boundary: B,
    cleanup: PerfCleanupState,
    event: PerfEvent,
) -> Outcome<OwnedPerfFd<B>> {
    let fd = match boundary.open(&PerfEventAttr::for_event(event)) {
        Ok(fd) => fd,
        Err(errno) => return classify_perf_open_errno(errno),
    };
    let owner = OwnedPerfFd {
        fd: Some(fd),
        boundary,
        cleanup: cleanup.clone(),
        event,
    };
    for request in [PERF_EVENT_IOC_RESET, PERF_EVENT_IOC_ENABLE] {
        if let Err(error) = boundary.ioctl(fd, request) {
            let cleanup_after_drop = cleanup.clone();
            drop(owner);
            return if cleanup_after_drop.cleanup_failed() {
                classify_sticky_cleanup(&cleanup_after_drop)
            } else {
                classify_perf_boundary_error(error)
            };
        }
    }
    Outcome::Success(owner)
}

fn stop_one_perf<B: PerfBoundary>(boundary: B, owner: &OwnedPerfFd<B>) -> Outcome<PerfCounter> {
    let fd = owner.fd.expect("owned descriptor is present");
    if let Err(error) = boundary.ioctl(fd, PERF_EVENT_IOC_DISABLE) {
        return classify_perf_boundary_error(error);
    }
    let mut value = PerfSnapshot::default();
    match boundary.read(fd, &mut value) {
        Ok(PERF_READ_BYTES) => Outcome::Success(perf_counter(owner.event, value)),
        Ok(other) => Outcome::Error(ErrorReason::PerfShortRead(other as isize)),
        Err(BoundaryReadError::Unexpected(count)) => {
            Outcome::Error(ErrorReason::PerfShortRead(count))
        }
        Err(BoundaryReadError::Errno(errno)) => classify_perf_boundary_errno(errno),
    }
}

fn finish_perf_session<B: PerfBoundary>(
    boundary: B,
    owners: &mut Vec<OwnedPerfFd<B>>,
) -> Outcome<[PerfCounter; 4]> {
    let mut observations = Vec::with_capacity(4);
    let mut failure = None;
    while let Some(owner) = owners.pop() {
        let event = owner.event;
        let fd = owner.fd.expect("owned descriptor is present");
        if let Err(error) = boundary.ioctl(fd, PERF_EVENT_IOC_DISABLE) {
            if failure.is_none() {
                failure = Some(classify_perf_boundary_error(error));
            }
        } else {
            let mut value = PerfSnapshot::default();
            match boundary.read(fd, &mut value) {
                Ok(PERF_READ_BYTES) => {
                    if failure.is_none() {
                        observations.push(perf_counter(event, value));
                    }
                }
                Ok(other) => {
                    if failure.is_none() {
                        failure = Some(Outcome::Error(ErrorReason::PerfShortRead(other as isize)));
                    }
                }
                Err(BoundaryReadError::Unexpected(count)) => {
                    if failure.is_none() {
                        failure = Some(Outcome::Error(ErrorReason::PerfShortRead(count)));
                    }
                }
                Err(BoundaryReadError::Errno(errno)) => {
                    if failure.is_none() {
                        failure = Some(classify_perf_boundary_errno(errno));
                    }
                }
            }
        }
        if let Err(error) = owner.finalize() {
            failure = Some(classify_perf_close_error(error));
        }
    }
    if let Some(failure) = failure {
        return failure;
    }
    observations.reverse();
    match observations.try_into() {
        Ok(values) => Outcome::Success(values),
        Err(_) => Outcome::Error(ErrorReason::PerfLifecycle),
    }
}

fn drop_owners_reverse<B: PerfBoundary>(owners: &mut Vec<OwnedPerfFd<B>>) {
    while let Some(owner) = owners.pop() {
        drop(owner);
    }
}

fn perf_counter(event: PerfEvent, value: PerfSnapshot) -> PerfCounter {
    PerfCounter {
        event,
        raw_count: value.raw_count,
        time_enabled_ns: value.time_enabled_ns,
        time_running_ns: value.time_running_ns,
        multiplexed: value.time_running_ns < value.time_enabled_ns,
        scaled_count: scale_perf_count(value),
    }
}

fn scale_perf_count(value: PerfSnapshot) -> Outcome<u64> {
    if value.time_enabled_ns == 0
        || value.time_running_ns == 0
        || value.time_running_ns > value.time_enabled_ns
    {
        return Outcome::Error(ErrorReason::PerfInvalidTime);
    }
    let numerator = u128::from(value.raw_count).checked_mul(u128::from(value.time_enabled_ns));
    let rounded =
        numerator.and_then(|number| number.checked_add(u128::from(value.time_running_ns / 2)));
    match rounded
        .map(|number| number / u128::from(value.time_running_ns))
        .and_then(|number| u64::try_from(number).ok())
    {
        Some(scaled) => Outcome::Success(scaled),
        None => Outcome::Overflow(OverflowReason::PerfScaling),
    }
}

pub fn validate_perf_progress(previous: PerfSnapshot, current: PerfSnapshot) -> Outcome<()> {
    if current.raw_count < previous.raw_count
        || current.time_enabled_ns < previous.time_enabled_ns
        || current.time_running_ns < previous.time_running_ns
    {
        Outcome::Error(ErrorReason::PerfDecrease)
    } else {
        Outcome::Success(())
    }
}

fn classify_perf_open_errno<T>(errno: i32) -> Outcome<T> {
    match errno {
        EPERM | EACCES => Outcome::Permission(errno),
        ENOENT | ENXIO | ENODEV | ENOSYS | EOPNOTSUPP => {
            Outcome::Unavailable(UnavailableReason::Interface(errno))
        }
        EOVERFLOW => Outcome::Overflow(OverflowReason::PerfErrno(errno)),
        _ => Outcome::Error(ErrorReason::Errno(errno)),
    }
}

fn classify_perf_boundary_errno<T>(errno: i32) -> Outcome<T> {
    match errno {
        EPERM | EACCES => Outcome::Permission(errno),
        EOVERFLOW => Outcome::Overflow(OverflowReason::PerfErrno(errno)),
        _ => Outcome::Error(ErrorReason::Errno(errno)),
    }
}

fn classify_perf_boundary_error<T>(error: BoundaryCallError) -> Outcome<T> {
    match error {
        BoundaryCallError::Errno(errno) => classify_perf_boundary_errno(errno),
        BoundaryCallError::Unexpected(result) => {
            Outcome::Error(ErrorReason::PerfUnexpectedReturn(result))
        }
    }
}

fn classify_perf_close_error<T>(error: BoundaryCallError) -> Outcome<T> {
    match error {
        BoundaryCallError::Errno(errno @ (EPERM | EACCES)) => Outcome::Permission(errno),
        BoundaryCallError::Errno(EOVERFLOW) => {
            Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW))
        }
        BoundaryCallError::Errno(errno) => Outcome::Error(ErrorReason::PerfCleanup(errno)),
        BoundaryCallError::Unexpected(result) => {
            Outcome::Error(ErrorReason::PerfCleanupUnexpected(result))
        }
    }
}

fn classify_sticky_cleanup<T>(cleanup: &PerfCleanupState) -> Outcome<T> {
    match cleanup.failure.get().expect("cleanup failure is present") {
        BoundaryCallError::Errno(errno @ (EPERM | EACCES)) => Outcome::Permission(errno),
        BoundaryCallError::Errno(EOVERFLOW) => {
            Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW))
        }
        BoundaryCallError::Errno(errno) => Outcome::Error(ErrorReason::PerfCleanup(errno)),
        BoundaryCallError::Unexpected(result) => {
            Outcome::Error(ErrorReason::PerfCleanupUnexpected(result))
        }
    }
}

pub fn clock_time(clock: Clock) -> Outcome<i128> {
    let mut value = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: This is the single authorized clock_gettime call; the pointer targets writable storage.
    let status = unsafe { clock_gettime(clock.id(), &mut value) };
    if status == 0 {
        convert_timespec(value)
    } else {
        classify_clock_errno(last_errno())
    }
}

pub fn clock_resolution(clock: Clock) -> Outcome<i128> {
    let mut value = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: This is the single authorized clock_getres call; the pointer targets writable storage.
    let status = unsafe { clock_getres(clock.id(), &mut value) };
    if status == 0 {
        convert_timespec(value)
    } else {
        classify_clock_errno(last_errno())
    }
}

pub fn resource_usage(scope: ResourceScope) -> Outcome<ResourceUsage> {
    let mut value = Rusage {
        ru_utime: Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_stime: Timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        ru_maxrss: 0,
        _pad_40: [0; 24],
        ru_minflt: 0,
        ru_majflt: 0,
        _pad_80: [0; 8],
        ru_inblock: 0,
        ru_oublock: 0,
        _pad_104: [0; 24],
        ru_nvcsw: 0,
        ru_nivcsw: 0,
    };
    // SAFETY: This is the single authorized getrusage call; the pointer targets writable storage.
    let status = unsafe { getrusage(scope.id(), &mut value) };
    if status == 0 {
        convert_rusage(value)
    } else {
        classify_rusage_errno(last_errno())
    }
}

pub fn open_file_length(file: &(impl AsRawFd + ?Sized)) -> Outcome<FileLength> {
    file_length_for_fd(file.as_raw_fd())
}

fn file_length_for_fd(fd: RawFd) -> Outcome<FileLength> {
    let mut value = Statx {
        stx_mask: 0,
        _pad_4: [0; 36],
        stx_size: 0,
        _pad_48: [0; 208],
    };
    // SAFETY: This is the single authorized statx call, with a NUL empty path and writable zeroed ABI storage.
    let status = unsafe {
        statx(
            fd,
            c"".as_ptr().cast(),
            AT_EMPTY_PATH | AT_STATX_SYNC_AS_STAT,
            STATX_SIZE,
            &mut value,
        )
    };
    if status == 0 {
        return convert_statx(value);
    }
    let errno = last_errno();
    if !statx_fallback_permitted(errno) {
        return classify_statx_errno(errno);
    }
    fstat_file_length(fd)
}

const fn statx_fallback_permitted(errno: i32) -> bool {
    errno == ENOSYS
}

fn fstat_file_length(fd: RawFd) -> Outcome<FileLength> {
    let mut value = Stat {
        _pad_0: [0; 48],
        st_size: 0,
        _pad_56: [0; 88],
    };
    // SAFETY: This is the single authorized fstat call after statx ENOSYS; storage is writable.
    let status = unsafe { fstat(fd, &mut value) };
    if status == 0 {
        convert_fstat(value)
    } else {
        classify_fstat_errno(last_errno())
    }
}

fn last_errno() -> i32 {
    io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

fn convert_timespec(value: Timespec) -> Outcome<i128> {
    if !(0..1_000_000_000).contains(&value.tv_nsec) {
        return Outcome::Error(ErrorReason::InvalidFraction);
    }
    match i128::from(value.tv_sec)
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(i128::from(value.tv_nsec)))
    {
        Some(value) => Outcome::Success(value),
        None => Outcome::Overflow(OverflowReason::Arithmetic),
    }
}

fn convert_timeval(value: Timeval) -> Outcome<i128> {
    if !(0..1_000_000).contains(&value.tv_usec) {
        return Outcome::Error(ErrorReason::InvalidFraction);
    }
    match i128::from(value.tv_sec)
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(i128::from(value.tv_usec) * 1_000))
    {
        Some(value) => Outcome::Success(value),
        None => Outcome::Overflow(OverflowReason::Arithmetic),
    }
}

fn nonnegative(value: i64) -> Result<u64, ErrorReason> {
    u64::try_from(value).map_err(|_| ErrorReason::NegativeCounter)
}

fn convert_rusage(value: Rusage) -> Outcome<ResourceUsage> {
    let Outcome::Success(user_nanoseconds) = convert_timeval(value.ru_utime) else {
        return convert_timeval(value.ru_utime).map_type();
    };
    let Outcome::Success(system_nanoseconds) = convert_timeval(value.ru_stime) else {
        return convert_timeval(value.ru_stime).map_type();
    };
    let counters = [
        value.ru_maxrss,
        value.ru_minflt,
        value.ru_majflt,
        value.ru_inblock,
        value.ru_oublock,
        value.ru_nvcsw,
        value.ru_nivcsw,
    ];
    let converted = counters.map(nonnegative);
    let [
        Ok(maxrss),
        Ok(minflt),
        Ok(majflt),
        Ok(inblock),
        Ok(oublock),
        Ok(nvcsw),
        Ok(nivcsw),
    ] = converted
    else {
        return Outcome::Error(ErrorReason::NegativeCounter);
    };
    let Some(maximum_resident_bytes) = maxrss.checked_mul(1024) else {
        return Outcome::Overflow(OverflowReason::Arithmetic);
    };
    Outcome::Success(ResourceUsage {
        user_nanoseconds,
        system_nanoseconds,
        maximum_resident_bytes,
        minor_faults: minflt,
        major_faults: majflt,
        input_blocks: inblock,
        output_blocks: oublock,
        voluntary_context_switches: nvcsw,
        involuntary_context_switches: nivcsw,
    })
}

impl<T> Outcome<T> {
    fn map_type<U>(self) -> Outcome<U> {
        match self {
            Self::Success(_) => unreachable!("called only for a non-success outcome"),
            Self::Unavailable(reason) => Outcome::Unavailable(reason),
            Self::Permission(errno) => Outcome::Permission(errno),
            Self::Overflow(reason) => Outcome::Overflow(reason),
            Self::Error(reason) => Outcome::Error(reason),
        }
    }
}

fn convert_statx(value: Statx) -> Outcome<FileLength> {
    if value.stx_mask & STATX_SIZE == 0 {
        return Outcome::Unavailable(UnavailableReason::MissingStatxSize);
    }
    let Ok(bytes) = i64::try_from(value.stx_size) else {
        return Outcome::Overflow(OverflowReason::FileSize);
    };
    Outcome::Success(FileLength {
        bytes,
        source: FileLengthSource::Statx,
        statx_only_fields: Ok(()),
    })
}

fn convert_fstat(value: Stat) -> Outcome<FileLength> {
    if value.st_size < 0 {
        return Outcome::Error(ErrorReason::NegativeFileSize);
    }
    Outcome::Success(FileLength {
        bytes: value.st_size,
        source: FileLengthSource::FstatFallback,
        statx_only_fields: Err(UnavailableReason::StatxOnlyAfterFstat),
    })
}

fn classify_clock_errno<T>(errno: i32) -> Outcome<T> {
    match errno {
        EINVAL | ENODEV | ENOSYS => Outcome::Unavailable(UnavailableReason::Interface(errno)),
        EACCES | EPERM => Outcome::Permission(errno),
        _ => Outcome::Error(ErrorReason::Errno(errno)),
    }
}

fn classify_rusage_errno<T>(errno: i32) -> Outcome<T> {
    match errno {
        EINVAL | ENOSYS => Outcome::Unavailable(UnavailableReason::Interface(errno)),
        EACCES | EPERM => Outcome::Permission(errno),
        _ => Outcome::Error(ErrorReason::Errno(errno)),
    }
}

fn classify_statx_errno<T>(errno: i32) -> Outcome<T> {
    match errno {
        EACCES | EPERM => Outcome::Permission(errno),
        EOVERFLOW => Outcome::Overflow(OverflowReason::FileSize),
        _ => Outcome::Error(ErrorReason::Errno(errno)),
    }
}

fn classify_fstat_errno<T>(errno: i32) -> Outcome<T> {
    match errno {
        EACCES | EPERM => Outcome::Permission(errno),
        EOVERFLOW => Outcome::Overflow(OverflowReason::FileSize),
        ENOSYS => Outcome::Unavailable(UnavailableReason::Interface(errno)),
        _ => Outcome::Error(ErrorReason::Errno(errno)),
    }
}

pub fn parse_statm(input: &str) -> Outcome<Statm> {
    if !input.is_ascii() {
        return Outcome::Error(ErrorReason::Parse(ParseReason::NonAscii));
    }
    let line = input.strip_suffix('\n').unwrap_or(input);
    if line.is_empty() || line.contains(['\n', '\r']) {
        return Outcome::Error(ErrorReason::Parse(ParseReason::LineCount));
    }
    let tokens: Vec<_> = line.split_ascii_whitespace().collect();
    if tokens.len() != 7 {
        return Outcome::Error(ErrorReason::Parse(ParseReason::TokenCount));
    }
    let mut values = [0; 7];
    for (destination, token) in values.iter_mut().zip(tokens) {
        let Outcome::Success(value) = parse_unsigned(token) else {
            return parse_unsigned(token).map_type();
        };
        *destination = value;
    }
    Outcome::Success(Statm {
        size: values[0],
        resident: values[1],
        shared: values[2],
        text: values[3],
        lib: values[4],
        data: values[5],
        dt: values[6],
    })
}

pub fn parse_status(input: &str) -> Outcome<StatusMemory> {
    if !input.is_ascii() {
        return Outcome::Error(ErrorReason::Parse(ParseReason::NonAscii));
    }
    let mut rss = None;
    let mut hwm = None;
    for line in input.lines() {
        let Some((key, value)) = line.split_once(':') else {
            return Outcome::Error(ErrorReason::Parse(ParseReason::MalformedLine));
        };
        if key.is_empty() {
            return Outcome::Error(ErrorReason::Parse(ParseReason::MalformedLine));
        }
        let slot = match key {
            "VmRSS" => &mut rss,
            "VmHWM" => &mut hwm,
            _ => continue,
        };
        if slot.is_some() {
            return Outcome::Error(ErrorReason::Parse(ParseReason::DuplicateField));
        }
        let tokens: Vec<_> = value.split_ascii_whitespace().collect();
        if tokens.len() != 2 {
            return Outcome::Error(ErrorReason::Parse(ParseReason::TokenCount));
        }
        if tokens[1] != "kB" {
            return Outcome::Error(ErrorReason::Parse(ParseReason::InvalidUnit));
        }
        let Outcome::Success(kib) = parse_unsigned(tokens[0]) else {
            return parse_unsigned(tokens[0]).map_type();
        };
        let Some(bytes) = kib.checked_mul(1024) else {
            return Outcome::Overflow(OverflowReason::Arithmetic);
        };
        *slot = Some(bytes);
    }
    match (rss, hwm) {
        (Some(resident_bytes), Some(high_water_bytes)) => Outcome::Success(StatusMemory {
            resident_bytes,
            high_water_bytes,
        }),
        _ => Outcome::Error(ErrorReason::Parse(ParseReason::MissingField)),
    }
}

pub fn parse_io(input: &str) -> Outcome<ProcessIo> {
    if !input.is_ascii() {
        return Outcome::Error(ErrorReason::Parse(ParseReason::NonAscii));
    }
    let names = [
        "rchar",
        "wchar",
        "syscr",
        "syscw",
        "read_bytes",
        "write_bytes",
        "cancelled_write_bytes",
    ];
    let mut values = [None; 7];
    for line in input.lines() {
        let Some((key, value)) = line.split_once(':') else {
            return Outcome::Error(ErrorReason::Parse(ParseReason::MalformedLine));
        };
        if key.is_empty() {
            return Outcome::Error(ErrorReason::Parse(ParseReason::MalformedLine));
        }
        let Some(index) = names.iter().position(|candidate| *candidate == key) else {
            continue;
        };
        if values[index].is_some() {
            return Outcome::Error(ErrorReason::Parse(ParseReason::DuplicateField));
        }
        let tokens: Vec<_> = value.split_ascii_whitespace().collect();
        if tokens.len() != 1 {
            return Outcome::Error(ErrorReason::Parse(ParseReason::TrailingToken));
        }
        let Outcome::Success(number) = parse_unsigned(tokens[0]) else {
            return parse_unsigned(tokens[0]).map_type();
        };
        values[index] = Some(number);
    }
    let [
        Some(rchar),
        Some(wchar),
        Some(syscr),
        Some(syscw),
        Some(read_bytes),
        Some(write_bytes),
        Some(cancelled_write_bytes),
    ] = values
    else {
        return Outcome::Error(ErrorReason::Parse(ParseReason::MissingField));
    };
    Outcome::Success(ProcessIo {
        rchar,
        wchar,
        syscr,
        syscw,
        read_bytes,
        write_bytes,
        cancelled_write_bytes,
    })
}

fn parse_unsigned(token: &str) -> Outcome<u64> {
    if token.starts_with(['+', '-']) {
        return Outcome::Error(ErrorReason::Parse(ParseReason::SignedValue));
    }
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return Outcome::Error(ErrorReason::Parse(ParseReason::InvalidNumber));
    }
    match token.parse() {
        Ok(value) => Outcome::Success(value),
        Err(_) => Outcome::Overflow(OverflowReason::NumericField),
    }
}

pub fn read_statm() -> Outcome<Statm> {
    read_text("/proc/self/statm", parse_statm)
}
pub fn read_status() -> Outcome<StatusMemory> {
    read_text("/proc/self/status", parse_status)
}
pub fn read_io() -> Outcome<ProcessIo> {
    read_text("/proc/self/io", parse_io)
}

fn read_text<T>(path: &str, parser: fn(&str) -> Outcome<T>) -> Outcome<T> {
    match fs::read_to_string(path) {
        Ok(input) => parser(&input),
        Err(error) => match error.kind() {
            io::ErrorKind::PermissionDenied => {
                Outcome::Permission(error.raw_os_error().unwrap_or(EACCES))
            }
            io::ErrorKind::NotFound => Outcome::Unavailable(UnavailableReason::NotFound),
            io::ErrorKind::Unsupported => Outcome::Unavailable(UnavailableReason::Unsupported),
            io::ErrorKind::InvalidData => Outcome::Error(ErrorReason::InvalidUtf8),
            kind => Outcome::Error(ErrorReason::Io(kind)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::mem::{align_of, offset_of, size_of};

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum Call {
        Open(PerfEventAttr),
        Ioctl(RawFd, u64),
        Read(RawFd),
        Close(RawFd),
    }

    struct FakeBoundary {
        opens: RefCell<VecDeque<Result<RawFd, i32>>>,
        ioctls: RefCell<VecDeque<Result<(), BoundaryCallError>>>,
        reads: RefCell<VecDeque<Result<PerfSnapshot, BoundaryReadError>>>,
        closes: RefCell<VecDeque<Result<(), BoundaryCallError>>>,
        calls: RefCell<Vec<Call>>,
    }

    impl FakeBoundary {
        fn successful() -> Self {
            Self {
                opens: RefCell::new((10..14).map(Ok).collect()),
                ioctls: RefCell::new((0..12).map(|_| Ok(())).collect()),
                reads: RefCell::new(
                    (1..=4)
                        .rev()
                        .map(|raw_count| {
                            Ok(PerfSnapshot {
                                raw_count,
                                time_enabled_ns: 10,
                                time_running_ns: 10,
                            })
                        })
                        .collect(),
                ),
                closes: RefCell::new((0..4).map(|_| Ok(())).collect()),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl PerfBoundary for &FakeBoundary {
        fn open(&self, attr: &PerfEventAttr) -> Result<RawFd, i32> {
            self.calls.borrow_mut().push(Call::Open(*attr));
            self.opens.borrow_mut().pop_front().unwrap()
        }

        fn ioctl(&self, fd: RawFd, request: u64) -> Result<(), BoundaryCallError> {
            self.calls.borrow_mut().push(Call::Ioctl(fd, request));
            self.ioctls.borrow_mut().pop_front().unwrap()
        }

        fn read(&self, fd: RawFd, result: &mut PerfSnapshot) -> Result<usize, BoundaryReadError> {
            self.calls.borrow_mut().push(Call::Read(fd));
            match self.reads.borrow_mut().pop_front().unwrap() {
                Ok(value) => {
                    *result = value;
                    Ok(PERF_READ_BYTES)
                }
                Err(error) => Err(error),
            }
        }

        fn close(&self, fd: RawFd) -> Result<(), BoundaryCallError> {
            self.calls.borrow_mut().push(Call::Close(fd));
            self.closes.borrow_mut().pop_front().unwrap()
        }
    }

    #[test]
    fn perf_constants_layout_selectors_and_zero_reserved_fields_are_exact() {
        assert_eq!(PERF_EVENT_OPEN_SYSCALL, 298);
        assert_eq!(PERF_FLAG_FD_CLOEXEC, 1 << 3);
        assert_eq!(PERF_ATTR_SIZE_VER0, 64);
        assert_eq!(
            (
                PERF_FORMAT_TOTAL_TIME_ENABLED,
                PERF_FORMAT_TOTAL_TIME_RUNNING
            ),
            (1, 2)
        );
        assert_eq!(PERF_READ_FORMAT, 3);
        assert_eq!(
            (
                PERF_EVENT_IOC_RESET,
                PERF_EVENT_IOC_ENABLE,
                PERF_EVENT_IOC_DISABLE
            ),
            (0x2403, 0x2400, 0x2401)
        );
        assert_eq!(size_of::<PerfEventAttr>(), 64);
        assert_eq!(align_of::<PerfEventAttr>(), 8);
        assert_eq!(
            (
                offset_of!(PerfEventAttr, event_type),
                offset_of!(PerfEventAttr, size),
                offset_of!(PerfEventAttr, config),
                offset_of!(PerfEventAttr, sample_period_or_freq),
                offset_of!(PerfEventAttr, sample_type),
                offset_of!(PerfEventAttr, read_format),
                offset_of!(PerfEventAttr, flags),
                offset_of!(PerfEventAttr, wakeup_events_or_watermark),
                offset_of!(PerfEventAttr, bp_type),
                offset_of!(PerfEventAttr, config1_or_bp_addr)
            ),
            (0, 4, 8, 16, 24, 32, 40, 48, 52, 56)
        );
        assert_eq!(size_of::<PerfSnapshot>(), 24);
        assert_eq!(align_of::<PerfSnapshot>(), 8);
        assert_eq!(
            (
                offset_of!(PerfSnapshot, raw_count),
                offset_of!(PerfSnapshot, time_enabled_ns),
                offset_of!(PerfSnapshot, time_running_ns)
            ),
            (0, 8, 16)
        );
        for (event, selector) in PerfEvent::ALL
            .into_iter()
            .zip([(0, 0), (0, 1), (1, 2), (1, 3)])
        {
            assert_eq!(event.selector(), selector);
            let attr = PerfEventAttr::for_event(event);
            assert_eq!((attr.size, attr.read_format, attr.flags), (64, 3, 1));
            assert_eq!(
                (
                    attr.sample_period_or_freq,
                    attr.sample_type,
                    attr.wakeup_events_or_watermark,
                    attr.bp_type,
                    attr.config1_or_bp_addr
                ),
                (0, 0, 0, 0, 0)
            );
        }
    }

    #[test]
    fn perf_lifecycle_is_independent_and_closes_in_reverse_order() {
        let fake = FakeBoundary::successful();
        let cleanup = PerfCleanupState::default();
        let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
            panic!()
        };
        let Outcome::Success(values) = finish_perf_session(&fake, &mut owners) else {
            panic!()
        };
        assert_eq!(
            values.map(|value| (value.event, value.raw_count)),
            [
                (PerfEvent::CpuCycles, 1),
                (PerfEvent::Instructions, 2),
                (PerfEvent::PageFaults, 3),
                (PerfEvent::ContextSwitches, 4),
            ]
        );
        let calls = fake.calls.borrow();
        for (index, event) in PerfEvent::ALL.into_iter().enumerate() {
            let start = index * 3;
            assert!(
                matches!(calls[start], Call::Open(attr) if attr == PerfEventAttr::for_event(event))
            );
            assert_eq!(
                calls[start + 1],
                Call::Ioctl(10 + index as i32, PERF_EVENT_IOC_RESET)
            );
            assert_eq!(
                calls[start + 2],
                Call::Ioctl(10 + index as i32, PERF_EVENT_IOC_ENABLE)
            );
        }
        assert_eq!(
            &calls[12..],
            &[
                Call::Ioctl(13, PERF_EVENT_IOC_DISABLE),
                Call::Read(13),
                Call::Close(13),
                Call::Ioctl(12, PERF_EVENT_IOC_DISABLE),
                Call::Read(12),
                Call::Close(12),
                Call::Ioctl(11, PERF_EVENT_IOC_DISABLE),
                Call::Read(11),
                Call::Close(11),
                Call::Ioctl(10, PERF_EVENT_IOC_DISABLE),
                Call::Read(10),
                Call::Close(10),
            ]
        );
        assert!(!cleanup.cleanup_failed());
    }

    #[test]
    fn per_event_production_path_orders_scales_stops_once_and_closes_once() {
        let fake = FakeBoundary::successful();
        let cleanup = PerfCleanupState::default();
        let Outcome::Success(owner) =
            open_one_perf(&fake, cleanup.clone(), PerfEvent::Instructions)
        else {
            panic!()
        };
        let mut stopped = false;
        let Outcome::Success(counter) = stop_event_session(&fake, &owner, &mut stopped) else {
            panic!()
        };
        assert_eq!(counter.event, PerfEvent::Instructions);
        assert_eq!(counter.scaled_count, Outcome::Success(counter.raw_count));
        assert_eq!(
            stop_event_session(&fake, &owner, &mut stopped),
            Outcome::Error(ErrorReason::PerfLifecycle)
        );
        assert_eq!(owner.finalize(), Ok(()));
        assert_eq!(
            &*fake.calls.borrow(),
            &[
                Call::Open(PerfEventAttr::for_event(PerfEvent::Instructions)),
                Call::Ioctl(10, PERF_EVENT_IOC_RESET),
                Call::Ioctl(10, PERF_EVENT_IOC_ENABLE),
                Call::Ioctl(10, PERF_EVENT_IOC_DISABLE),
                Call::Read(10),
                Call::Close(10),
            ]
        );
        assert!(!cleanup.cleanup_failed());
    }

    #[test]
    fn per_event_failures_are_classified_and_cleanup_is_isolated() {
        for (errno, expected) in [
            (EPERM, Outcome::Permission(EPERM)),
            (EACCES, Outcome::Permission(EACCES)),
            (
                ENOENT,
                Outcome::Unavailable(UnavailableReason::Interface(ENOENT)),
            ),
            (
                ENXIO,
                Outcome::Unavailable(UnavailableReason::Interface(ENXIO)),
            ),
            (
                ENODEV,
                Outcome::Unavailable(UnavailableReason::Interface(ENODEV)),
            ),
            (
                ENOSYS,
                Outcome::Unavailable(UnavailableReason::Interface(ENOSYS)),
            ),
            (
                EOPNOTSUPP,
                Outcome::Unavailable(UnavailableReason::Interface(EOPNOTSUPP)),
            ),
            (
                EOVERFLOW,
                Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW)),
            ),
            (EINVAL, Outcome::Error(ErrorReason::Errno(EINVAL))),
            (ENOMEM, Outcome::Error(ErrorReason::Errno(ENOMEM))),
            (EBUSY, Outcome::Error(ErrorReason::Errno(EBUSY))),
            (EMFILE, Outcome::Error(ErrorReason::Errno(EMFILE))),
            (ENFILE, Outcome::Error(ErrorReason::Errno(ENFILE))),
        ] {
            let fake = FakeBoundary {
                opens: RefCell::new(VecDeque::from([Err(errno)])),
                ..FakeBoundary::successful()
            };
            assert_eq!(
                open_one_perf(&fake, PerfCleanupState::default(), PerfEvent::CpuCycles)
                    .map_type::<()>(),
                expected
            );
        }

        for failure in [
            BoundaryCallError::Errno(EBUSY),
            BoundaryCallError::Unexpected(7),
        ] {
            let fake = FakeBoundary::successful();
            fake.ioctls.borrow_mut()[0] = Err(failure);
            let cleanup = PerfCleanupState::default();
            let result =
                open_one_perf(&fake, cleanup.clone(), PerfEvent::CpuCycles).map_type::<()>();
            assert_eq!(
                result,
                match failure {
                    BoundaryCallError::Errno(e) => Outcome::Error(ErrorReason::Errno(e)),
                    BoundaryCallError::Unexpected(v) =>
                        Outcome::Error(ErrorReason::PerfUnexpectedReturn(v)),
                }
            );
            assert!(!cleanup.cleanup_failed());
        }

        let first = FakeBoundary::successful();
        first.ioctls.borrow_mut()[0] = Err(BoundaryCallError::Errno(EBUSY));
        first.closes.borrow_mut()[0] = Err(BoundaryCallError::Errno(EBADF));
        assert_eq!(
            open_one_perf(&first, PerfCleanupState::default(), PerfEvent::CpuCycles)
                .map_type::<()>(),
            Outcome::Error(ErrorReason::PerfCleanup(EBADF))
        );
        let second = FakeBoundary::successful();
        assert!(matches!(
            open_one_perf(
                &second,
                PerfCleanupState::default(),
                PerfEvent::Instructions
            ),
            Outcome::Success(_)
        ));
    }

    #[test]
    fn per_event_stop_and_close_failures_cover_all_boundary_shapes() {
        for (failure, expected) in [
            (BoundaryCallError::Errno(EPERM), Outcome::Permission(EPERM)),
            (
                BoundaryCallError::Unexpected(8),
                Outcome::Error(ErrorReason::PerfUnexpectedReturn(8)),
            ),
        ] {
            let fake = FakeBoundary::successful();
            fake.ioctls.borrow_mut()[2] = Err(failure);
            let Outcome::Success(owner) =
                open_one_perf(&fake, PerfCleanupState::default(), PerfEvent::CpuCycles)
            else {
                panic!()
            };
            assert_eq!(stop_one_perf(&fake, &owner), expected);
        }
        for (read, expected) in [
            (
                Err(BoundaryReadError::Errno(EOVERFLOW)),
                Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW)),
            ),
            (
                Err(BoundaryReadError::Unexpected(23)),
                Outcome::Error(ErrorReason::PerfShortRead(23)),
            ),
        ] {
            let fake = FakeBoundary::successful();
            fake.reads.borrow_mut()[0] = read;
            let Outcome::Success(owner) =
                open_one_perf(&fake, PerfCleanupState::default(), PerfEvent::PageFaults)
            else {
                panic!()
            };
            assert_eq!(stop_one_perf(&fake, &owner), expected);
        }
        for (failure, expected) in [
            (
                BoundaryCallError::Errno(EBADF),
                Outcome::Error(ErrorReason::PerfCleanup(EBADF)),
            ),
            (
                BoundaryCallError::Unexpected(9),
                Outcome::Error(ErrorReason::PerfCleanupUnexpected(9)),
            ),
        ] {
            let fake = FakeBoundary::successful();
            fake.closes.borrow_mut()[0] = Err(failure);
            let Outcome::Success(owner) = open_one_perf(
                &fake,
                PerfCleanupState::default(),
                PerfEvent::ContextSwitches,
            ) else {
                panic!()
            };
            assert_eq!(
                owner
                    .finalize()
                    .map_or_else(classify_perf_close_error, |_| Outcome::Success(())),
                expected
            );
            assert_eq!(
                fake.calls
                    .borrow()
                    .iter()
                    .filter(|c| matches!(c, Call::Close(_)))
                    .count(),
                1
            );
        }
    }

    #[test]
    fn perf_errno_classes_are_exact_and_never_retry() {
        for errno in [EPERM, EACCES] {
            assert_eq!(
                classify_perf_open_errno::<()>(errno),
                Outcome::Permission(errno)
            );
        }
        for errno in [ENOENT, ENXIO, ENODEV, ENOSYS, EOPNOTSUPP] {
            assert_eq!(
                classify_perf_open_errno::<()>(errno),
                Outcome::Unavailable(UnavailableReason::Interface(errno))
            );
        }
        assert_eq!(
            classify_perf_open_errno::<()>(EOVERFLOW),
            Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW))
        );
        for errno in [4, EBADF, ENOMEM, EBUSY, EINVAL, EMFILE, ENFILE] {
            assert_eq!(
                classify_perf_open_errno::<()>(errno),
                Outcome::Error(ErrorReason::Errno(errno))
            );
        }
        assert_eq!(
            classify_perf_boundary_errno::<()>(ENOSYS),
            Outcome::Error(ErrorReason::Errno(ENOSYS))
        );
        let fake = FakeBoundary {
            opens: RefCell::new(VecDeque::from([Err(4)])),
            ..FakeBoundary::successful()
        };
        assert!(matches!(
            open_perf_session(&fake, &PerfCleanupState::default()),
            Outcome::Error(ErrorReason::Errno(4))
        ));
        assert_eq!(fake.calls.borrow().len(), 1);
    }

    #[test]
    fn perf_scaling_retains_raw_times_multiplexing_and_checks_edges() {
        let counter = perf_counter(
            PerfEvent::CpuCycles,
            PerfSnapshot {
                raw_count: 5,
                time_enabled_ns: 3,
                time_running_ns: 2,
            },
        );
        assert_eq!(
            (
                counter.raw_count,
                counter.time_enabled_ns,
                counter.time_running_ns,
                counter.multiplexed
            ),
            (5, 3, 2, true)
        );
        assert_eq!(counter.scaled_count, Outcome::Success(8));
        assert_eq!(
            scale_perf_count(PerfSnapshot {
                raw_count: 7,
                time_enabled_ns: 9,
                time_running_ns: 9
            }),
            Outcome::Success(7)
        );
        assert_eq!(
            scale_perf_count(PerfSnapshot {
                raw_count: 1,
                time_enabled_ns: 3,
                time_running_ns: 2
            }),
            Outcome::Success(2)
        );
        for value in [
            PerfSnapshot {
                raw_count: 1,
                time_enabled_ns: 0,
                time_running_ns: 0,
            },
            PerfSnapshot {
                raw_count: 1,
                time_enabled_ns: 1,
                time_running_ns: 0,
            },
            PerfSnapshot {
                raw_count: 1,
                time_enabled_ns: 1,
                time_running_ns: 2,
            },
        ] {
            assert_eq!(
                scale_perf_count(value),
                Outcome::Error(ErrorReason::PerfInvalidTime)
            );
        }
        assert_eq!(
            scale_perf_count(PerfSnapshot {
                raw_count: u64::MAX,
                time_enabled_ns: u64::MAX,
                time_running_ns: 1
            }),
            Outcome::Overflow(OverflowReason::PerfScaling)
        );
    }

    #[test]
    fn perf_rejects_decrease_in_every_field() {
        let base = PerfSnapshot {
            raw_count: 5,
            time_enabled_ns: 6,
            time_running_ns: 4,
        };
        assert_eq!(validate_perf_progress(base, base), Outcome::Success(()));
        for current in [
            PerfSnapshot {
                raw_count: 4,
                ..base
            },
            PerfSnapshot {
                time_enabled_ns: 5,
                ..base
            },
            PerfSnapshot {
                time_running_ns: 3,
                ..base
            },
        ] {
            assert_eq!(
                validate_perf_progress(base, current),
                Outcome::Error(ErrorReason::PerfDecrease)
            );
        }
    }

    #[test]
    fn perf_short_read_lifecycle_and_cleanup_fail_closed() {
        for count in [0, 1, 23, 25] {
            let fake = FakeBoundary::successful();
            fake.reads.borrow_mut()[0] = Err(BoundaryReadError::Unexpected(count));
            let cleanup = PerfCleanupState::default();
            let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
                panic!()
            };
            assert_eq!(
                finish_perf_session(&fake, &mut owners),
                Outcome::Error(ErrorReason::PerfShortRead(count))
            );
        }
        let fake = FakeBoundary::successful();
        fake.ioctls.borrow_mut()[0] = Err(BoundaryCallError::Errno(EINVAL));
        let cleanup = PerfCleanupState::default();
        assert_eq!(
            open_perf_session(&fake, &cleanup).map_type::<()>(),
            Outcome::Error(ErrorReason::Errno(EINVAL))
        );
        assert_eq!(
            &*fake.calls.borrow(),
            &[
                Call::Open(PerfEventAttr::for_event(PerfEvent::CpuCycles)),
                Call::Ioctl(10, PERF_EVENT_IOC_RESET),
                Call::Close(10)
            ]
        );

        let fake = FakeBoundary::successful();
        fake.closes.borrow_mut()[0] = Err(BoundaryCallError::Errno(EBADF));
        let cleanup = PerfCleanupState::default();
        let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
            panic!()
        };
        assert_eq!(
            finish_perf_session(&fake, &mut owners),
            Outcome::Error(ErrorReason::PerfCleanup(EBADF))
        );
        assert_eq!(cleanup.cleanup_errno(), Some(EBADF));
    }

    #[test]
    fn perf_close_and_sticky_cleanup_preserve_specific_errno_classes() {
        for (errno, expected) in [
            (EPERM, Outcome::Permission(EPERM)),
            (EACCES, Outcome::Permission(EACCES)),
            (
                EOVERFLOW,
                Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW)),
            ),
        ] {
            let fake = FakeBoundary::successful();
            fake.closes.borrow_mut()[0] = Err(BoundaryCallError::Errno(errno));
            let cleanup = PerfCleanupState::default();
            let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
                panic!()
            };
            assert_eq!(finish_perf_session(&fake, &mut owners), expected);
            assert_eq!(cleanup.cleanup_errno(), Some(errno));

            let fake = FakeBoundary::successful();
            fake.ioctls.borrow_mut()[0] = Err(BoundaryCallError::Errno(EINVAL));
            fake.closes.borrow_mut()[0] = Err(BoundaryCallError::Errno(errno));
            let cleanup = PerfCleanupState::default();
            assert_eq!(
                open_perf_session(&fake, &cleanup).map_type::<()>(),
                expected.clone().map_type()
            );
            assert_eq!(cleanup.cleanup_errno(), Some(errno));
            assert_eq!(
                &*fake.calls.borrow(),
                &[
                    Call::Open(PerfEventAttr::for_event(PerfEvent::CpuCycles)),
                    Call::Ioctl(10, PERF_EVENT_IOC_RESET),
                    Call::Close(10),
                ]
            );
        }
    }

    #[test]
    fn every_perf_lifecycle_failure_is_terminal_without_retry() {
        let fake = FakeBoundary::successful();
        fake.ioctls.borrow_mut()[1] = Err(BoundaryCallError::Errno(EBUSY));
        let cleanup = PerfCleanupState::default();
        assert_eq!(
            open_perf_session(&fake, &cleanup).map_type::<()>(),
            Outcome::Error(ErrorReason::Errno(EBUSY))
        );
        assert_eq!(
            fake.calls
                .borrow()
                .iter()
                .filter(|call| matches!(call, Call::Ioctl(_, PERF_EVENT_IOC_ENABLE)))
                .count(),
            1
        );

        let fake = FakeBoundary::successful();
        fake.ioctls.borrow_mut()[8] = Err(BoundaryCallError::Errno(EPERM));
        let cleanup = PerfCleanupState::default();
        let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
            panic!()
        };
        assert_eq!(
            finish_perf_session(&fake, &mut owners),
            Outcome::Permission(EPERM)
        );

        let fake = FakeBoundary::successful();
        fake.reads.borrow_mut()[0] = Err(BoundaryReadError::Errno(EOVERFLOW));
        let cleanup = PerfCleanupState::default();
        let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
            panic!()
        };
        assert_eq!(
            finish_perf_session(&fake, &mut owners),
            Outcome::Overflow(OverflowReason::PerfErrno(EOVERFLOW))
        );
        assert_eq!(
            fake.calls
                .borrow()
                .iter()
                .filter(|call| matches!(call, Call::Read(13)))
                .count(),
            1
        );
    }

    #[test]
    fn perf_unwind_drop_closes_once_and_marks_sticky_failure() {
        let fake = FakeBoundary::successful();
        fake.closes.borrow_mut()[0] = Err(BoundaryCallError::Errno(EBADF));
        let cleanup = PerfCleanupState::default();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Outcome::Success(fds) = open_perf_session(&fake, &cleanup) else {
                panic!()
            };
            let _session = PerfFdOwners {
                fds,
                lifetime: std::marker::PhantomData,
            };
            panic!("synthetic unwind");
        }));
        assert!(result.is_err());
        assert_eq!(cleanup.cleanup_errno(), Some(EBADF));
        assert_eq!(
            fake.calls
                .borrow()
                .iter()
                .filter_map(|call| match call {
                    Call::Close(fd) => Some(*fd),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [13, 12, 11, 10]
        );
    }

    #[test]
    fn positive_ioctl_and_close_returns_are_invalid_without_reading_errno() {
        let errno_reads = std::cell::Cell::new(0);
        let stale_errno = || {
            errno_reads.set(errno_reads.get() + 1);
            EBADF
        };
        assert_eq!(
            classify_zero_result(7, stale_errno),
            Err(BoundaryCallError::Unexpected(7))
        );
        assert_eq!(errno_reads.get(), 0);
        assert_eq!(
            classify_perf_boundary_error::<()>(BoundaryCallError::Unexpected(7)),
            Outcome::Error(ErrorReason::PerfUnexpectedReturn(7))
        );
        assert_eq!(
            classify_perf_close_error::<()>(BoundaryCallError::Unexpected(9)),
            Outcome::Error(ErrorReason::PerfCleanupUnexpected(9))
        );

        let fake = FakeBoundary::successful();
        fake.ioctls.borrow_mut()[0] = Err(BoundaryCallError::Unexpected(7));
        assert_eq!(
            open_perf_session(&fake, &PerfCleanupState::default()).map_type::<()>(),
            Outcome::Error(ErrorReason::PerfUnexpectedReturn(7))
        );

        let fake = FakeBoundary::successful();
        fake.closes.borrow_mut()[0] = Err(BoundaryCallError::Unexpected(9));
        let cleanup = PerfCleanupState::default();
        let Outcome::Success(mut owners) = open_perf_session(&fake, &cleanup) else {
            panic!()
        };
        assert_eq!(
            finish_perf_session(&fake, &mut owners),
            Outcome::Error(ErrorReason::PerfCleanupUnexpected(9))
        );
        assert!(cleanup.cleanup_failed());
        assert_eq!(cleanup.cleanup_errno(), None);
        assert_eq!(cleanup.cleanup_unexpected_return(), Some(9));
    }

    #[test]
    fn frozen_constants_and_selectors() {
        assert_eq!((CLOCK_REALTIME, CLOCK_MONOTONIC_RAW), (0, 4));
        assert_eq!((RUSAGE_SELF, RUSAGE_THREAD), (0, 1));
        assert_eq!(
            (AT_EMPTY_PATH, AT_STATX_SYNC_AS_STAT, STATX_SIZE),
            (0x1000, 0, 0x200)
        );
        assert_eq!(
            (
                EPERM, ENOENT, EBADF, EACCES, ENODEV, ENOTDIR, EINVAL, ENOSYS, EOVERFLOW
            ),
            (1, 2, 9, 13, 19, 20, 22, 38, 75)
        );
        assert_eq!((Clock::Realtime.id(), Clock::MonotonicRaw.id()), (0, 4));
        assert_eq!(
            (ResourceScope::Process.id(), ResourceScope::Thread.id()),
            (0, 1)
        );
        assert_eq!(Clock::try_from(0), Ok(Clock::Realtime));
        assert_eq!(Clock::try_from(4), Ok(Clock::MonotonicRaw));
        assert_eq!(Clock::try_from(1), Err(1));
        assert_eq!(ResourceScope::try_from(0), Ok(ResourceScope::Process));
        assert_eq!(ResourceScope::try_from(1), Ok(ResourceScope::Thread));
        assert_eq!(ResourceScope::try_from(-1), Err(-1));
    }

    #[test]
    fn frozen_abi_layouts() {
        assert_eq!(
            (size_of::<i32>(), size_of::<i64>(), size_of::<u64>()),
            (4, 8, 8)
        );
        assert_eq!(
            (
                size_of::<Timespec>(),
                align_of::<Timespec>(),
                offset_of!(Timespec, tv_sec),
                offset_of!(Timespec, tv_nsec)
            ),
            (16, 8, 0, 8)
        );
        assert_eq!(
            (
                size_of::<Timeval>(),
                align_of::<Timeval>(),
                offset_of!(Timeval, tv_sec),
                offset_of!(Timeval, tv_usec)
            ),
            (16, 8, 0, 8)
        );
        assert_eq!((size_of::<Rusage>(), align_of::<Rusage>()), (144, 8));
        assert_eq!(
            (
                offset_of!(Rusage, ru_utime),
                offset_of!(Rusage, ru_stime),
                offset_of!(Rusage, ru_maxrss),
                offset_of!(Rusage, ru_minflt),
                offset_of!(Rusage, ru_majflt),
                offset_of!(Rusage, ru_inblock),
                offset_of!(Rusage, ru_oublock),
                offset_of!(Rusage, ru_nvcsw),
                offset_of!(Rusage, ru_nivcsw)
            ),
            (0, 16, 32, 64, 72, 88, 96, 128, 136)
        );
        assert_eq!(
            (
                size_of::<Statx>(),
                align_of::<Statx>(),
                offset_of!(Statx, stx_mask),
                offset_of!(Statx, stx_size)
            ),
            (256, 8, 0, 40)
        );
        assert_eq!(
            (
                size_of::<Stat>(),
                align_of::<Stat>(),
                offset_of!(Stat, st_size)
            ),
            (144, 8, 48)
        );
    }

    #[test]
    fn errno_classification_is_exact() {
        for errno in [EINVAL, ENODEV, ENOSYS] {
            assert!(matches!(
                classify_clock_errno::<()>(errno),
                Outcome::Unavailable(_)
            ));
        }
        for errno in [EACCES, EPERM] {
            assert_eq!(
                classify_clock_errno::<()>(errno),
                Outcome::Permission(errno)
            );
        }
        assert_eq!(
            classify_clock_errno::<()>(4),
            Outcome::Error(ErrorReason::Errno(4))
        );
        for errno in [EINVAL, ENOSYS] {
            assert_eq!(
                classify_rusage_errno::<()>(errno),
                Outcome::Unavailable(UnavailableReason::Interface(errno))
            );
        }
        for errno in [EACCES, EPERM] {
            assert_eq!(
                classify_rusage_errno::<()>(errno),
                Outcome::Permission(errno)
            );
            assert_eq!(
                classify_statx_errno::<()>(errno),
                Outcome::Permission(errno)
            );
            assert_eq!(
                classify_fstat_errno::<()>(errno),
                Outcome::Permission(errno)
            );
        }
        assert_eq!(
            classify_rusage_errno::<()>(ENODEV),
            Outcome::Error(ErrorReason::Errno(ENODEV))
        );
        assert_eq!(
            classify_statx_errno::<()>(EOVERFLOW),
            Outcome::Overflow(OverflowReason::FileSize)
        );
        assert_eq!(
            classify_statx_errno::<()>(ENOSYS),
            Outcome::Error(ErrorReason::Errno(ENOSYS))
        );
        assert_eq!(
            classify_statx_errno::<()>(EBADF),
            Outcome::Error(ErrorReason::Errno(EBADF))
        );
        for errno in [
            EPERM, ENOENT, EBADF, EACCES, ENODEV, ENOTDIR, EINVAL, EOVERFLOW,
        ] {
            assert!(!statx_fallback_permitted(errno));
        }
        assert!(statx_fallback_permitted(ENOSYS));
        assert!(matches!(
            classify_fstat_errno::<()>(ENOSYS),
            Outcome::Unavailable(_)
        ));
        assert_eq!(
            classify_fstat_errno::<()>(EOVERFLOW),
            Outcome::Overflow(OverflowReason::FileSize)
        );
        assert_eq!(
            classify_fstat_errno::<()>(EBADF),
            Outcome::Error(ErrorReason::Errno(EBADF))
        );
    }

    #[test]
    fn time_and_counter_conversion_is_checked() {
        assert_eq!(
            convert_timespec(Timespec {
                tv_sec: -1,
                tv_nsec: 1
            }),
            Outcome::Success(-999_999_999)
        );
        assert!(matches!(
            convert_timespec(Timespec {
                tv_sec: 0,
                tv_nsec: -1
            }),
            Outcome::Error(ErrorReason::InvalidFraction)
        ));
        assert!(matches!(
            convert_timespec(Timespec {
                tv_sec: 0,
                tv_nsec: 1_000_000_000
            }),
            Outcome::Error(ErrorReason::InvalidFraction)
        ));
        assert_eq!(
            convert_timeval(Timeval {
                tv_sec: 2,
                tv_usec: 3
            }),
            Outcome::Success(2_000_003_000)
        );
        let mut usage = Rusage {
            ru_utime: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: Timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 1,
            _pad_40: [0; 24],
            ru_minflt: 2,
            ru_majflt: 3,
            _pad_80: [0; 8],
            ru_inblock: 4,
            ru_oublock: 5,
            _pad_104: [0; 24],
            ru_nvcsw: 6,
            ru_nivcsw: 7,
        };
        assert!(matches!(
            convert_rusage(usage),
            Outcome::Success(ResourceUsage {
                maximum_resident_bytes: 1024,
                ..
            })
        ));
        usage.ru_minflt = -1;
        assert_eq!(
            convert_rusage(usage),
            Outcome::Error(ErrorReason::NegativeCounter)
        );
        usage.ru_minflt = 0;
        usage.ru_maxrss = i64::MAX;
        assert_eq!(
            convert_rusage(usage),
            Outcome::Overflow(OverflowReason::Arithmetic)
        );
    }

    #[test]
    fn file_results_fail_closed() {
        let missing = Statx {
            stx_mask: 0,
            _pad_4: [0; 36],
            stx_size: 0,
            _pad_48: [0; 208],
        };
        assert_eq!(
            convert_statx(missing),
            Outcome::Unavailable(UnavailableReason::MissingStatxSize)
        );
        let too_large = Statx {
            stx_mask: STATX_SIZE,
            stx_size: u64::MAX,
            ..missing
        };
        assert_eq!(
            convert_statx(too_large),
            Outcome::Overflow(OverflowReason::FileSize)
        );
        let fallback = convert_fstat(Stat {
            _pad_0: [0; 48],
            st_size: 0,
            _pad_56: [0; 88],
        });
        assert!(matches!(
            fallback,
            Outcome::Success(FileLength {
                bytes: 0,
                statx_only_fields: Err(UnavailableReason::StatxOnlyAfterFstat),
                ..
            })
        ));
        assert_eq!(
            convert_fstat(Stat {
                _pad_0: [0; 48],
                st_size: -1,
                _pad_56: [0; 88]
            }),
            Outcome::Error(ErrorReason::NegativeFileSize)
        );
    }

    #[test]
    fn procfs_valid_examples_are_exact() {
        assert_eq!(
            parse_statm("1 2 3 4 5 6 7\n"),
            Outcome::Success(Statm {
                size: 1,
                resident: 2,
                shared: 3,
                text: 4,
                lib: 5,
                data: 6,
                dt: 7
            })
        );
        assert_eq!(
            parse_status("Name: x\nVmRSS: 2 kB\nVmHWM: 3 kB\n"),
            Outcome::Success(StatusMemory {
                resident_bytes: 2048,
                high_water_bytes: 3072
            })
        );
        assert_eq!(
            parse_io(
                "rchar: 1\nwchar: 2\nsyscr: 3\nsyscw: 4\nread_bytes: 5\nwrite_bytes: 6\ncancelled_write_bytes: 7\n"
            ),
            Outcome::Success(ProcessIo {
                rchar: 1,
                wchar: 2,
                syscr: 3,
                syscw: 4,
                read_bytes: 5,
                write_bytes: 6,
                cancelled_write_bytes: 7
            })
        );
    }

    #[test]
    fn procfs_rejects_all_malformed_classes() {
        for input in [
            "",
            "1 2 3 4 5 6",
            "1 2 3 4 5 6 7 8",
            "+1 2 3 4 5 6 7",
            "x 2 3 4 5 6 7",
            "1 2 3 4 5 6 18446744073709551616",
            "1 2 3 4 5 6 7\n8 9 0 1 2 3 4\n",
            "1 2 3 4 5 6 é",
        ] {
            assert!(!matches!(parse_statm(input), Outcome::Success(_)));
        }
        for input in [
            "VmRSS: 1 kB\n",
            "VmRSS: 1 kB\nVmRSS: 2 kB\nVmHWM: 3 kB\n",
            "VmRSS: -1 kB\nVmHWM: 3 kB\n",
            "VmRSS: 1 KB\nVmHWM: 3 kB\n",
            "VmRSS: 1 kB extra\nVmHWM: 3 kB\n",
            "VmRSS: 18014398509481984 kB\nVmHWM: 3 kB\n",
            "VmRSS: 1 kB\nVmHWM: é kB\n",
        ] {
            assert!(!matches!(parse_status(input), Outcome::Success(_)));
        }
        let base = "rchar: 1\nwchar: 2\nsyscr: 3\nsyscw: 4\nread_bytes: 5\nwrite_bytes: 6\ncancelled_write_bytes: 7\n";
        for input in [
            base.replace("rchar: 1\n", ""),
            format!("{base}rchar: 8\n"),
            base.replace("rchar: 1", "rchar: -1"),
            base.replace("rchar: 1", "rchar: x"),
            base.replace("rchar: 1", "rchar: 1 kB"),
            base.replace("rchar: 1", "rchar: 18446744073709551616"),
            base.replace("rchar: 1", "rchar: é"),
        ] {
            assert!(!matches!(parse_io(&input), Outcome::Success(_)));
        }
    }

    #[test]
    fn unavailable_is_not_numeric_zero_and_equality_is_deterministic() {
        let value: Outcome<u64> = Outcome::Unavailable(UnavailableReason::NotFound);
        assert_ne!(value, Outcome::Success(0));
        assert_eq!(parse_statm("1 2 3 4 5 6 7"), parse_statm("1 2 3 4 5 6 7"));
    }
}
