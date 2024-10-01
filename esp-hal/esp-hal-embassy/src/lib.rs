#![no_std]

pub use macros::main;

#[cfg(feature = "executors")]
pub use self::executor::Executor;

#[cfg(feature = "executors")]
mod executor;
