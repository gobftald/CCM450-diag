#![no_std]

#[macro_use(unwrap)]
extern crate console;

// 14
mod duration;
mod instant;
mod timer;

// 30
pub use duration::Duration;
pub use embassy_time_driver::TICK_HZ;

// 32
pub use instant::Instant;

// 33
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
