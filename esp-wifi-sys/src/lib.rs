#![no_std]

// 5
pub mod c_types;

// 11
#[cfg_attr(feature = "esp32c3", path = "include/esp32c3.rs")]
pub mod include;
