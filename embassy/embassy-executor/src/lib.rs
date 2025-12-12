#![no_std]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

//#[macro_use(panic, unwrap)]
//extern crate console;

//12
pub use embassy_executor_macros::task;

// 42
pub mod raw;

// 50
mod spawner;
pub use spawner::*;
