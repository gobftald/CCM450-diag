//! # Reading of eFuses (ESP32-C3)
//!
//! ## Overview
//!
//! The `efuse` module provides functionality for reading eFuse data
//! from the `ESP32-C3` chip, allowing access to various chip-specific
//! information such as:
//!
//!   * MAC address
//!   * ADC calibration data
//!
//! and more. It is useful for retrieving chip-specific configuration and
//! identification data during runtime.
//!
//! The `Efuse` struct represents the eFuse peripheral and is responsible for
//! reading various eFuse fields and values.
//!

// 44
pub use self::fields::*;
use crate::{peripherals::EFUSE, soc::efuse_field::EfuseField};
// 47
mod fields;

/// A struct representing the eFuse functionality of the chip.
// 50
pub struct Efuse;

#[derive(Debug, Clone, Copy, strum::FromRepr)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u32)]
// 147
pub(crate) enum EfuseBlock {
    //Block0,
    Block1 = 1,
    //Block2,
    //Block3,
    //Block4,
    //Block5,
    //Block6,
    //Block7,
    //Block8,
    //Block9,
    //Block10,
}

// 161
impl EfuseBlock {
    pub(crate) fn address(self) -> *const u32 {
        let efuse = EFUSE::regs();
        match self {
            Self::Block1 => efuse.rd_mac_spi_sys_0().as_ptr(),
        }
    }
}
