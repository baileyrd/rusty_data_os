//! R29-authorized, Linux/x86_64-only preflight interfaces.
//!
//! This library does not assemble or execute a descriptive D1 harness.

#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("exp1-descriptive-d1-harness supports only Linux on x86_64");

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(unsafe_code)]
pub mod linux_capture;
