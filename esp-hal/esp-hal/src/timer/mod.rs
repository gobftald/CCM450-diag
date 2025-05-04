//! # General-purpose Timers
//!
//! ## Overview
//! The [OneShotTimer] and [PeriodicTimer] types can be backed by any hardware
//! peripheral which implements the [Timer] trait. This means that the same API
//! can be used to interact with different hardware timers, like the `TIMG` and
//! SYSTIMER.

// 61
#[cfg(systimer)]
pub mod systimer;
