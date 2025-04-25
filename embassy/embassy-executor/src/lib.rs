#![no_std]
#![feature(local_waker)]

#[macro_use(panic)]
extern crate console;

//12
pub use embassy_executor_macros::task;

#[cfg_attr(target_arch = "riscv32", path = "arch/riscv32.rs")]
// 42
mod arch;

#[cfg(target_arch = "riscv32")]
// 46
pub use arch::*;

pub mod raw;

// 52
mod spawner;
pub use spawner::*;
