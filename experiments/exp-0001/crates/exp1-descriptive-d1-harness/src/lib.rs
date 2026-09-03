//! Deterministic R31 observation assembly and the R29/R30 Linux ABI types.
//!
//! There is deliberately no live harness entry point.  [`orchestration`] only
//! operates on caller-injected boundaries.

#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("exp1-descriptive-d1-harness supports only Linux on x86_64");

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(unsafe_code)]
pub mod linux_capture;

pub mod orchestration;
