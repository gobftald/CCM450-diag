//! # Timekeeping
//!
//! This module provides types for representing frequency and duration, as well
//! as an instant in time. Time is measured since boot, and can be accessed
//! by the [`Instant::now`] function.

// 7
use core::fmt::{Debug, Formatter, Result as FmtResult};

// 12
type InnerRate = fugit::Rate<u32, 1, 1>;
type InnerInstant = fugit::Instant<u64, 1, 1_000_000>;
type InnerDuration = fugit::Duration<u64, 1, 1_000_000>;

/// Represents a rate or frequency of events.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 18
pub struct Rate(InnerRate);

// 34
impl Debug for Rate {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Rate({} Hz)", self.as_hz())
    }
}

#[cfg(feature = "defmt")]
// 42
impl defmt::Format for Rate {
    #[inline]
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(f, "{=u32} Hz", self.as_hz())
    }
}

// 49
impl Rate {
    /// Shorthand for creating a rate which represents hertz.
    #[inline]
    // 52
    pub const fn from_hz(val: u32) -> Self {
        Self(InnerRate::Hz(val))
    }

    /// Shorthand for creating a rate which represents megahertz.
    #[inline]
    // 64
    pub const fn from_mhz(val: u32) -> Self {
        Self(InnerRate::MHz(val))
    }

    /// Convert the `Rate` to an interger number of Hz.
    #[inline]
    // 70
    pub const fn as_hz(&self) -> u32 {
        self.0.to_Hz()
    }

    /// Convert the `Rate` to an interger number of MHz.
    #[inline]
    // 82
    pub const fn as_mhz(&self) -> u32 {
        self.0.to_MHz()
    }

    /// Convert the `Rate` to a `Duration`.
    #[inline]
    pub const fn as_duration(&self) -> Duration {
        Duration::from_micros(1_000_000 / self.as_hz() as u64)
    }
}

// Represents an instant in time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 122
pub struct Instant(InnerInstant);

// 165
impl Instant {
    /// Represents the moment the system booted.
    // 167
    pub const EPOCH: Instant = Instant(InnerInstant::from_ticks(0));

    /// Returns the current instant.
    ///
    /// The counter won’t measure time in sleep-mode.
    ///
    /// The timer has a 1 microsecond resolution and will wrap after
    #[inline]
    // 178
    pub fn now() -> Self {
        now()
    }

    #[inline]
    // 183
    pub(crate) fn from_ticks(ticks: u64) -> Self {
        Instant(InnerInstant::from_ticks(ticks))
    }

    /// Returns the elapsed `Duration` since boot.
    #[inline]
    // 189
    pub fn duration_since_epoch(&self) -> Duration {
        Self::EPOCH.elapsed()
    }

    /// Returns the elapsed `Duration` since this `Instant` was created.
    #[inline]
    // 195
    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }
}

// 209
impl core::ops::Sub for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}

/// Represents a duration of time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 243
pub struct Duration(InnerDuration);

// 245
impl Debug for Duration {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Duration({} µs)", self.as_micros())
    }
}

#[cfg(feature = "defmt")]
// 260
impl defmt::Format for Duration {
    #[inline]
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(f, "{=u64} µs", self.as_micros())
    }
}

// 274
impl Duration {
    /// A duration of zero time.
    // 276
    pub const ZERO: Self = Self(InnerDuration::from_ticks(0));

    /// Creates a duration which represents microseconds.
    #[inline]
    // 283
    pub const fn from_micros(val: u64) -> Self {
        Self(InnerDuration::micros(val))
    }

    // 311
    delegate::delegate! {
        #[inline]
        to self.0 {
            /// Convert the `Duration` to an interger number of microseconds.
            #[call(to_micros)]
            // 316
            pub const fn as_micros(&self) -> u64;
        }
    }
}

// 419
impl core::ops::Div<u32> for Duration {
    type Output = Self;

    #[inline]
    // 423
    fn div(self, rhs: u32) -> Self::Output {
        Duration(self.0 / rhs)
    }
}

#[inline]
// 438
fn now() -> Instant {
    // 463
    #[cfg(not(esp32))]
    let (ticks, div) = {
        use crate::timer::systimer::{SystemTimer, Unit};
        let ticks = SystemTimer::unit_value(Unit::Unit0);
        (ticks, (SystemTimer::ticks_per_second() / 1_000_000))
    };

    Instant::from_ticks(ticks / div)
}
