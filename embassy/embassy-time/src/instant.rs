// 2
use core::ops::Add;

// 4
use super::{Duration, GCD_1M, TICK_HZ};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// An Instant in time, based on the MCU's clock ticks since startup.
// 9
pub struct Instant {
    ticks: u64,
}

// 13
impl Instant {
    /// Returns an Instant representing the current time.
    #[inline]
    // 20
    pub fn now() -> Instant {
        Instant {
            ticks: embassy_time_driver::now(),
        }
    }

    /// Create an Instant from a microsecond count since system boot.
    // 32
    pub const fn from_micros(micros: u64) -> Self {
        Self {
            ticks: micros * (TICK_HZ / GCD_1M) / (1_000_000 / GCD_1M),
        }
    }

    /// Tick count since system boot.
    // 53
    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    /// Microseconds since system boot.
    // 68
    pub const fn as_micros(&self) -> u64 {
        self.ticks * (1_000_000 / GCD_1M) / (TICK_HZ / GCD_1M)
    }

    /// Adds one Duration to self, returning a new `Instant` or None in the event of an overflow.
    // 109
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.ticks
            .checked_add(duration.ticks)
            .map(|ticks| Instant { ticks })
    }
}

// 119
impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, other: Duration) -> Instant {
        //self.checked_add(other)
        //.expect("overflow when adding duration to instant")
        unwrap!(
            self.checked_add(other),
            "overflow when adding duration to instant"
        )
    }
}
