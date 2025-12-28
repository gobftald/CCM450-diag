//! # SOC (System-on-Chip) module (ESP32-C3)
//!
//! ## Overview
//!
//! The `SOC` module provides access, functions and structures that are useful
//! for interacting with various system-related peripherals on `ESP32-C3` chip.
//!
//! Also few constants are defined in this module for `ESP32-C3` chip:
//!    * I2S_SCLK: 160_000_000 - I2S clock frequency
//!    * I2S_DEFAULT_CLK_SRC: 2 - I2S clock source

// 13
pub mod efuse;

// 16
//pub mod gpio;
//pub mod peripherals; // moved into root folder
pub(crate) mod regi2c;

pub(crate) use esp32c3 as pac;

#[allow(unused)]
// 36
pub(crate) mod registers {
    pub const INTERRUPT_MAP_BASE: u32 = 0x600c2000;
}

// 41
pub(crate) mod constants {
    use crate::time::Rate;

    /// RC FAST Clock value (Hertz).
    // 59
    pub const RC_FAST_CLK: Rate = Rate::from_khz(17500);
}
