// 2
use core::ops::Add;

// 4
use super::Duration;

// 9
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// An Instant in time, based on the MCU's clock ticks since startup.
pub struct Instant {
    ticks: u64,
}

// 13
impl Instant {
    /// Returns an Instant representing the current time.
    #[inline]
    // 21
    pub fn now() -> Instant {
        Instant {
            ticks: embassy_time_driver::now(),
        }
    }

    /// Tick count since system boot.
    // 85
    pub const fn as_ticks(&self) -> u64 {
        self.ticks
    }

    /// Adds one Duration to self, returning a new `Instant` or None in the event of an overflow.
    // 141
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.ticks
            .checked_add(duration.ticks)
            .map(|ticks| Instant { ticks })
    }
}

// 163
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
