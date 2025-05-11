//! # CPU Clock Control
//!
//! ## Overview
//!
//! Clocks are mainly sourced from oscillator (OSC), RC, and PLL circuits, and
//! then processed by the dividers or selectors, which allows most functional
//! modules to select their working clock according to their power consumption
//! and performance requirements.
//!
//! The clock subsystem  is used to source and distribute system/module clocks
//! from a range of root clocks. The clock tree driver maintains the basic
//! functionality of the system clock and the intricate relationship among
//! module clocks.
//!
//! ## Configuration
//!
//! During HAL initialization, specify a CPU clock speed to configure the
//! desired clock frequencies.
//!
//! The `CPU clock` is responsible for defining the speed at which the central
//! processing unit (CPU) operates. This driver provides predefined options for
//! different CPU clock speeds, such as

// 49
use crate::time::Rate;

/// Clock properties
// 62
pub trait Clock {
    /// Frequency of the clock in [Rate].
    // 64
    fn frequency(&self) -> Rate;

    /// Frequency of the clock in Hertz
    // 72
    fn hz(&self) -> u32 {
        self.frequency().as_hz()
    }
}

/// CPU clock speed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum CpuClock {
    /// 80MHz CPU clock
    #[cfg(not(esp32h2))]
    _80MHz = 80,

    /// 96MHz CPU clock
    #[cfg(esp32h2)]
    _96MHz = 96,

    /// 120MHz CPU clock
    #[cfg(esp32c2)]
    _120MHz = 120,

    /// 160MHz CPU clock
    #[cfg(not(any(esp32c2, esp32h2)))]
    _160MHz = 160,

    /// 240MHz CPU clock
    #[cfg(xtensa)]
    _240MHz = 240,
}

// 107
impl Default for CpuClock {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(esp32h2)] {
                Self::_96MHz
            } else {
                // FIXME: I don't think this is correct in general?
                Self::_80MHz
            }
        }
    }
}

// 120
impl CpuClock {
    /// Use the highest possible frequency for a particular chip.
    pub const fn max() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(esp32c2)] {
                Self::_120MHz
            } else if #[cfg(any(esp32c3, esp32c6))] {
                Self::_160MHz
            } else if #[cfg(esp32h2)] {
                Self::_96MHz
            } else {
                Self::_240MHz
            }
        }
    }
}

/// XTAL clock speed
#[derive(/*Debug,*/ Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 147
pub enum XtalClock {
    /// 26MHz XTAL clock
    #[cfg(any(esp32, esp32c2))]
    _26M,
    /// 32MHz XTAL clock
    #[cfg(any(esp32c3, esp32h2, esp32s3))]
    _32M,
    /// 40MHz XTAL clock
    #[cfg(not(esp32h2))]
    _40M,
    /// Other XTAL clock
    Other(u32),
}

// 161
impl Clock for XtalClock {
    fn frequency(&self) -> Rate {
        match self {
            #[cfg(any(esp32, esp32c2))]
            XtalClock::_26M => Rate::from_mhz(26),
            #[cfg(any(esp32c3, esp32h2, esp32s3))]
            XtalClock::_32M => Rate::from_mhz(32),
            #[cfg(not(esp32h2))]
            XtalClock::_40M => Rate::from_mhz(40),
            XtalClock::Other(mhz) => Rate::from_mhz(*mhz),
        }
    }
}

/// Clock frequencies.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 258
pub struct Clocks {}

// 295
impl Clocks {
    /// Returns the xtal frequency.
    ///
    /// This function will run the frequency estimation if called before
    /// [`crate::init()`].
    #[cfg(systimer)]
    #[inline]
    // 321
    pub(crate) fn xtal_freq() -> Rate {
        /*
        if esp_config::esp_config_str!("ESP_HAL_CONFIG_XTAL_FREQUENCY") == "auto" {
            if let Some(clocks) = Self::try_get() {
                return clocks.xtal_clock;
            }
        }
        */

        Self::measure_xtal_frequency().frequency()
    }
}

#[cfg(esp32c3)]
// 438
impl Clocks {
    // 439
    fn measure_xtal_frequency() -> XtalClock {
        XtalClock::_40M
    }
}
