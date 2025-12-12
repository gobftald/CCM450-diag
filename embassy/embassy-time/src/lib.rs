#![no_std]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

//#[macro_use(unwrap)]
//extern crate console;

// 14
mod duration;
mod instant;
mod timer;

// 30
pub use duration::Duration;
pub use embassy_time_driver::TICK_HZ;
pub use instant::Instant;
pub use timer::Timer;

// 35
const fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// 43
pub(crate) const GCD_1K: u64 = gcd(TICK_HZ, 1_000);
pub(crate) const GCD_1M: u64 = gcd(TICK_HZ, 1_000_000);
