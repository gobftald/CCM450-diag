//! # Timekeeping
//!
//! This module provides types for representing frequency and duration, as well
//! as an instant in time. Time is measured since boot, and can be accessed
//! by the [`Instant::now`] function.

// 12
type InnerRate = fugit::Rate<u32, 1, 1>;
type InnerInstant = fugit::Instant<u64, 1, 1_000_000>;
type InnerDuration = fugit::Duration<u64, 1, 1_000_000>;

/// Represents a rate or frequency of events.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 18
pub struct Rate(InnerRate);

// 49
impl Rate {
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
}

// Represents an instant in time.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// 122
pub struct Instant(InnerInstant);

// 158
impl Instant {
    /// Represents the moment the system booted.
    // 160
    pub const EPOCH: Instant = Instant(InnerInstant::from_ticks(0));

    /// Returns the current instant.
    ///
    /// The counter won’t measure time in sleep-mode.
    ///
    /// The timer has a 1 microsecond resolution and will wrap after
    #[inline]
    // 171
    pub fn now() -> Self {
        now()
    }

    #[inline]
    // 176
    pub(crate) fn from_ticks(ticks: u64) -> Self {
        Instant(InnerInstant::from_ticks(ticks))
    }

    /// Returns the elapsed `Duration` since boot.
    #[inline]
    // 182
    pub fn duration_since_epoch(&self) -> Duration {
        Self::EPOCH.elapsed()
    }

    /// Returns the elapsed `Duration` since this `Instant` was created.
    #[inline]
    // 188
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
// 236
pub struct Duration(InnerDuration);

// 260
impl Duration {
    delegate::delegate! {
        #[inline]
        to self.0 {
            /// Convert the `Duration` to an interger number of microseconds.
            #[call(to_micros)]
            // 302
            pub const fn as_micros(&self) -> u64;
        }
    }
}

#[inline]
// 424
fn now() -> Instant {
    // 449
    #[cfg(not(esp32))]
    let (ticks, div) = {
        use crate::timer::systimer::{SystemTimer, Unit};
        let ticks = SystemTimer::unit_value(Unit::Unit0);
        (ticks, (SystemTimer::ticks_per_second() / 1_000_000))
    };

    Instant::from_ticks(ticks / div)
}
