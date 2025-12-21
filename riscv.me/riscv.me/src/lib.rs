//! Low level access to RISC-V processors
//!
//! # Optional features
//!
//! ## `s-mode`
//!
//! This feature re-exports in `interrupt` S-mode interrupt functions defined in `interrupt::supervisor`.
//! By default, the crate assumes that the target is running in M-mode.
//! Thus, `interrupt` re-exports the M-mode functions defined in `interrupt::machine`.
//!
//! ## `critical-section-single-hart`
//!
//! This feature enables a [`critical-section`](https://github.com/rust-embedded/critical-section)
//! implementation suitable for single-hart targets, based on disabling interrupts globally.
//! This feature uses S-mode interrupt handling if the `s-mode` feature is enabled, and M-mode otherwise.

#![no_std]
#![allow(clippy::missing_safety_doc)]

// 41
pub mod bits;

// 43
pub mod interrupt;
pub mod register;

// Re-export crates of the RISC-V ecosystem
// 49
pub use riscv_pac::*;

#[cfg(all(riscv, feature = "critical-section-single-hart"))]
// 57
mod critical_section;
