#![no_std]
#![feature(local_waker)]

#[macro_use(panic, unwrap)]
extern crate console;

//12
pub use embassy_executor_macros::task;

// 50
pub mod raw;

// 52
mod spawner;
pub use spawner::*;
