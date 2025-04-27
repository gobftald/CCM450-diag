//! Embassy support for [esp-hal].
//!
//! [Embassy] is a modern asynchronous framework intended for use with embedded
//! systems. This package provides support for building applications using
//! Embassy with [esp-hal].
//!
//! Note that this crate currently requires you to enable the `unstable` feature
//! on `esp-hal`.
//!
//! [esp-hal]: https://github.com/esp-rs/esp-hal
//! [embassy]: https://github.com/embassy-rs/embassy
//!
//! ## Executors
//!
//! Two types of executors are provided:
//!
//! - [Executor]: A thread-mode executor
//! - [InterruptExecutor]: An interrupt-mode executor
//!
//! [InterruptExecutor] can be used to achieve preemptive multitasking in
//! asynchronous applications, which is typically something reserved for more
//! traditional RTOS. More information can be found in the [Embassy
//! documentation].
//!
//! [embassy documentation]: https://embassy.dev/book/
//!
//! ## Initialization
//!
//! Embassy **must** be initialized by calling the [init] function. This
//! initialization must be performed *prior* to spawning any tasks.
//!
//! Initialization requires a number of timers to be passed in. The number of
//! timers required depends on the timer queue flavour used, as well as the
//! number of executors started. If you use the `multiple-integrated` timer
//! queue flavour, then you need to pass as many timers as you start executors.
//! In other cases, you can pass a single timer.
#![no_std]

// 58
pub use macros::embassy_main as main;

#[cfg(feature = "executors")]
// 61
pub use self::executor::Executor;

#[cfg(feature = "executors")]
// 65
pub(crate) mod executor;
