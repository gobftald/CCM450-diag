//! # GPIO configuration module (ESP32-C3)
//!
//! ## Overview
//!
//! The `GPIO` module provides functions and configurations for controlling the
//! `General Purpose Input/Output` pins on the `ESP32-C3` chip. It allows you to
//! configure pins as inputs or outputs, set their state and read their state.
//!
//! Let's get through the functionality and configurations provided by this GPIO
//! module:
//!   - `io_mux_reg(gpio_num: u8) -> &'static
//!     crate::peripherals::io_mux::GPIO0:`:
//!       * Returns the IO_MUX register for the specified GPIO pin number.

// 39
use crate::{gpio::AlternateFunction, pac::io_mux, peripherals::IO_MUX};

// 48
pub(crate) const FUNC_IN_SEL_OFFSET: usize = 0;

//50
pub(crate) type InputSignalType = u8;
pub(crate) type OutputSignalType = u8;
pub(crate) const OUTPUT_SIGNAL_MAX: u8 = 128;
pub(crate) const INPUT_SIGNAL_MAX: u8 = 100;

// 55
pub(crate) const ONE_INPUT: u8 = 0x1e;
pub(crate) const ZERO_INPUT: u8 = 0x1f;

// 58
pub(crate) const GPIO_FUNCTION: AlternateFunction = AlternateFunction::_1;

// 60
pub(crate) fn io_mux_reg(gpio_num: u8) -> &'static io_mux::GPIO {
    IO_MUX::regs().gpio(gpio_num as usize)
}

/// Peripheral input signals for the GPIO mux
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 73
pub enum InputSignal {
    U0RXD = 6,
    U1RXD = 9,
}

/// Peripheral output signals for the GPIO mux
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[doc(hidden)]
// 123
pub enum OutputSignal {
    U0TXD = 6,
    U1TXD = 9,
    GPIO = 128,
}
