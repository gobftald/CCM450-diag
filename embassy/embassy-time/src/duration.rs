// 4
use super::{GCD_1K, TICK_HZ};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Represents the difference between two [Instant](struct.Instant.html)s
// 10
pub struct Duration {
    pub(crate) ticks: u64,
}

// 14
impl Duration {
    /// Creates a duration from the specified number of milliseconds, rounding up.
    // 51
    pub const fn from_millis(millis: u64) -> Duration {
        Duration {
            ticks: div_ceil(millis * (TICK_HZ / GCD_1K), 1000 / GCD_1K),
        }
    }
}

// 276
#[inline]
const fn div_ceil(num: u64, den: u64) -> u64 {
    (num + den - 1) / den
}
