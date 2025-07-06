/*! Time structures.

The `time` module contains structures used to represent both
absolute and relative time.

 - [Instant] is used to represent absolute time.
 - [Duration] is used to represent relative time.
 */

/// A relative amount of time.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// 181
pub struct Duration {
    micros: u64,
}

// 185
impl Duration {
    /// The longest possible duration we can encode.
    // 188
    pub const MAX: Duration = Duration::from_micros(u64::MAX);

    /// Create a new `Duration` from a number of microseconds.
    // 190
    pub const fn from_micros(micros: u64) -> Duration {
        Duration { micros }
    }

    /// Create a new `Duration` from a number of seconds.
    // 202
    pub const fn from_secs(secs: u64) -> Duration {
        Duration {
            micros: secs * 1000000,
        }
    }
}
