#![no_std]

#[macro_use(assert_eq, unwrap, panic, trace, debug)]
extern crate console;

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

// 169
pub mod time;
pub mod wire;
