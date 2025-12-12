#![no_std]
#![allow(mismatched_lifetime_syntaxes)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

/*
#[allow(unused_imports)]
#[macro_use(
    assert,
    assert_eq,
    unwrap,
    panic,
    trace,
    debug,
    debug_assert,
    unreachable
)]
extern crate console;
*/

#[macro_use]
// 128
mod macros;

// 130
mod rand;

// 133
pub mod config {
    // 140
    pub const IFACE_MAX_ADDR_COUNT: usize = 8;
    // 142
    pub const IFACE_MAX_ROUTE_COUNT: usize = 4;
    // 144
    pub const IFACE_NEIGHBOR_CACHE_COUNT: usize = 3;
}

#[cfg(feature = "medium-ethernet")]
// 163
pub mod iface;

// 165
pub mod phy;

#[cfg(feature = "socket")]
// 167
pub mod socket;
pub mod storage;
pub mod time;
pub mod wire;
