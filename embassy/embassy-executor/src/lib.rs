#![no_std]

// for defmt logs (brings defmt runtime into scope)
use esp_println as _;

// 12
pub use embassy_executor_macros::task;

// 40
pub mod raw;

// 42
mod spawner;
pub use spawner::*;
