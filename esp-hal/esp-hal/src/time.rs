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

    /// Shorthand for creating a rate which represents kilohertz.
    #[inline]
    // 58
    pub const fn from_khz(val: u32) -> Self {
        Self(InnerRate::kHz(val))
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

impl core::ops::Div for Rate {
    type Output = u32;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

// Represents an instant in time.
// 195
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(InnerInstant);

// 221
#[cfg(feature = "defmt")]
impl defmt::Format for Instant {
    #[inline]
    fn format(&self, f: defmt::Formatter<'_>) {
        defmt::write!(
            f,
            "{=u64} µs since epoch",
            self.duration_since_epoch().as_micros()
        )
    }
}

// 239
impl Instant {
    /// Represents the moment the system booted.
    pub const EPOCH: Instant = Instant(InnerInstant::from_ticks(0));

    /// Returns the current instant.
    ///
    /// The counter won’t measure time in sleep-mode.
    ///
    /// The timer has a 1 microsecond resolution and will wrap after
    // 265
    #[inline]
    pub fn now() -> Self {
        now()
    }

    // 270
    #[inline]
    pub(crate) fn from_ticks(ticks: u64) -> Self {
        Instant(InnerInstant::from_ticks(ticks))
    }

    /// Returns the elapsed `Duration` since boot.
    // 287
    #[inline]
    pub fn duration_since_epoch(&self) -> Duration {
        *self - Self::EPOCH 
    }

    /// Returns the elapsed `Duration` since this `Instant` was created.
    // 304
    #[inline]
    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }
}

// 216
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

    /// A duration representing the maximum possible time.
    pub const MAX: Self = Self(InnerDuration::from_ticks(u64::MAX));

    /// Creates a duration which represents microseconds.
    #[inline]
    // 283
    pub const fn from_micros(val: u64) -> Self {
        Self(InnerDuration::micros(val))
    }

    /// Creates a duration which represents seconds.
    #[inline]
    pub const fn from_secs(val: u64) -> Self {
        Self(InnerDuration::secs(val))
    }

    // 311
    delegate::delegate! {
        #[inline]
        to self.0 {
            /// Convert the `Duration` to an interger number of microseconds.
            #[call(to_micros)]
            // 316
            pub const fn as_micros(&self) -> u64;

            /// Convert the `Duration` to an interger number of milliseconds.
            #[call(to_millis)]
            pub const fn as_millis(&self) -> u64;
        }
    }
}

impl core::ops::AddAssign for Duration {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
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

impl core::ops::Add<Duration> for Instant {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        Instant(self.0 + rhs.0)
    }
}
