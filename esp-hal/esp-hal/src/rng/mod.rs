//! # Random Number Generator (RNG)
//!
//! ## Overview
//! The Random Number Generator (RNG) module provides an interface to generate
//! random numbers using the RNG peripheral on ESP chips. This driver allows you
//! to generate random numbers that can be used for various cryptographic,
//! security, or general-purpose applications.
//!
//! There are certain pre-conditions which must be met in order for the RNG to
//! produce *true* random numbers. The hardware RNG produces true random numbers
//! under any of the following conditions:
//!
//! - RF subsystem is enabled (i.e. Wi-Fi or Bluetooth are enabled).
//! - An internal entropy source has been enabled by calling
//!   `bootloader_random_enable()` and not yet disabled by calling
//!   `bootloader_random_disable()`.
//! - While the ESP-IDF Second stage bootloader is running. This is because the
//!   default ESP-IDF bootloader implementation calls
//!   `bootloader_random_enable()` when the bootloader starts, and
//!   `bootloader_random_disable()` before executing the app.
//!
//! When any of these conditions are true, samples of physical noise are
//! continuously mixed into the internal hardware RNG state to provide entropy.
//! If none of the above conditions are true, the output of the RNG should be
//! considered pseudo-random only.
//!
//! ## Configuration
//! To use the [Rng] Driver, you need to initialize it with the RNG peripheral.
//! Once initialized, you can generate random numbers by calling the `random`
//! method, which returns a 32-bit unsigned integer.

// 93
use crate::peripherals::RNG;

/// Random number generator driver
#[derive(Clone, Copy)]
#[non_exhaustive]
// 101
pub struct Rng;

// 103
impl Rng {
    /// Create a new random number generator instance
    // 105
    pub fn new(_rng: RNG<'_>) -> Self {
        Self
    }

    #[inline]
    /// Reads currently available `u32` integer from `RNG`
    // 111
    pub fn random(&mut self) -> u32 {
        // SAFETY: read-only register access
        RNG::regs().data().read().bits()
    }
}
