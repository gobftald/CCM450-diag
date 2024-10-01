#![no_std]

// 12
pub use embassy_executor_macros::task;

// 40
pub mod raw;

// 42
mod spawner;
pub use spawner::*;
