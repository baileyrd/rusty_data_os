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

unsafe extern "C" {
    fn clock_gettime(clock_id: i32, result: *mut Timespec) -> i32;
    fn clock_getres(clock_id: i32, result: *mut Timespec) -> i32;
    fn getrusage(who: i32, result: *mut Rusage) -> i32;
    fn statx(dirfd: i32, pathname: *const u8, flags: i32, mask: u32, result: *mut Statx) -> i32;
    fn fstat(fd: i32, result: *mut Stat) -> i32;
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

pub fn open_file_length(file: &impl AsRawFd) -> Outcome<FileLength> {
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
    use std::mem::{align_of, offset_of, size_of};

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
