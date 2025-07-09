/*! Time structures.

The `time` module contains structures used to represent both
absolute and relative time.

 - [Instant] is used to represent absolute time.
 - [Duration] is used to represent relative time.
 */

/// A representation of an absolute time value.
///
/// The `Instant` type is a wrapper around a `i64` value that
/// represents a number of microseconds, monotonically increasing
/// since an arbitrary moment in time, such as system startup.
///
/// * A value of `0` is inherently arbitrary.
/// * A value less than `0` indicates a time before the starting
///   point.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// 25
pub struct Instant {
    micros: i64,
}

// 29
impl Instant {
    /// Create a new `Instant` from a number of microseconds.
    // 33
    pub fn from_micros<T: Into<i64>>(micros: T) -> Instant {
        Instant {
            micros: micros.into(),
        }
    }

    /// Create a new `Instant` from a number of milliseconds.
    // 44
    pub fn from_millis<T: Into<i64>>(millis: T) -> Instant {
        Instant {
            micros: millis.into() * 1000,
        }
    }

    /// The fractional number of milliseconds that have passed
    /// since the beginning of time.
    // 77
    pub const fn millis(&self) -> i64 {
        self.micros % 1000000 / 1000
    }

    /// The number of whole seconds that have passed since the
    /// beginning of time.
    //89
    pub const fn secs(&self) -> i64 {
        self.micros / 1000000
    }
}

#[cfg(feature = "defmt")]
// 137
impl defmt::Format for Instant {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}.{:03}s", self.secs(), self.millis());
    }
}

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

    /// The fractional number of milliseconds in this `Duration`.
    // 209
    pub const fn millis(&self) -> u64 {
        self.micros / 1000 % 1000
    }

    /// The number of whole seconds in this `Duration`.
    // 219
    pub const fn secs(&self) -> u64 {
        self.micros / 1000000
    }
}

#[cfg(feature = "defmt")]
// 241
impl defmt::Format for Duration {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}.{:03}s", self.secs(), self.millis());
    }
}
