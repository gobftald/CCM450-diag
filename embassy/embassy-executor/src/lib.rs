#![no_std]

#[cfg(feature = "_arch")]
#[cfg_attr(feature = "arch-riscv32", path = "arch/riscv32.rs")]
// 42
mod arch;

#[cfg(feature = "_arch")]
// 46
pub use arch::*;

pub mod raw;
