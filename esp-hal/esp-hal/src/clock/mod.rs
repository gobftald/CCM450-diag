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

use core::{cell::Cell, marker::PhantomData};

#[cfg(any(bt, ieee802154, wifi))]
use esp_sync::RawMutex;

use crate::ESP_HAL_LOCK;
#[cfg(wifi)]
use crate::peripherals::WIFI;

// 49
use crate::time::Rate;

// 53
#[cfg_attr(esp32c3, path = "clocks_ll/esp32c3.rs")]
pub(crate) mod clocks_ll;

/// Clock properties
// 62
pub trait Clock {
    /// Frequency of the clock in [Rate].
    // 64
    fn frequency(&self) -> Rate;

    /// Frequency of the clock in Megahertz
    // 67
    fn mhz(&self) -> u32 {
        self.frequency().as_mhz()
    }

    /// Frequency of the clock in Hertz
    // 72
    fn hz(&self) -> u32 {
        self.frequency().as_hz()
    }
}

/// CPU clock speed
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 85
pub enum CpuClock {
    /// 80MHz CPU clock
    #[cfg(not(esp32h2))]
    #[default]
    _80MHz = 80,

    /// 96MHz CPU clock
    #[cfg(esp32h2)]
    #[default]
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

// 137
impl Clock for CpuClock {
    fn frequency(&self) -> Rate {
        Rate::from_mhz(*self as u32)
    }
}

/// XTAL clock speed
#[derive(Debug, Clone, Copy)]
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

#[allow(unused)]
// 177
pub(crate) enum PllClock {
    #[cfg(esp32h2)]
    Pll8MHz,
    #[cfg(any(esp32c6, esp32h2))]
    Pll48MHz,
    #[cfg(esp32h2)]
    Pll64MHz,
    #[cfg(esp32c6)]
    Pll80MHz,
    #[cfg(esp32h2)]
    Pll96MHz,
    #[cfg(esp32c6)]
    Pll120MHz,
    #[cfg(esp32c6)]
    Pll160MHz,
    #[cfg(esp32c6)]
    Pll240MHz,
    #[cfg(not(any(esp32c2, esp32c6, esp32h2)))]
    Pll320MHz,
    #[cfg(not(esp32h2))]
    Pll480MHz,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 229
pub(crate) enum ApbClock {
    #[cfg(esp32h2)]
    ApbFreq32MHz,
    #[cfg(not(esp32h2))]
    ApbFreq40MHz,
    #[cfg(not(esp32h2))]
    ApbFreq80MHz,
    ApbFreqOther(u32),
}

// 239
impl Clock for ApbClock {
    fn frequency(&self) -> Rate {
        match self {
            #[cfg(esp32h2)]
            ApbClock::ApbFreq32MHz => Rate::from_mhz(32),
            #[cfg(not(esp32h2))]
            ApbClock::ApbFreq40MHz => Rate::from_mhz(40),
            #[cfg(not(esp32h2))]
            ApbClock::ApbFreq80MHz => Rate::from_mhz(80),
            ApbClock::ApbFreqOther(mhz) => Rate::from_mhz(*mhz),
        }
    }
}

/// Clock frequencies.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 258
pub struct Clocks {
    /// CPU clock frequency
    pub cpu_clock: Rate,

    /// APB clock frequency
    pub apb_clock: Rate,

    /// XTAL clock frequency
    pub xtal_clock: Rate,
}

// 293
static mut ACTIVE_CLOCKS: Option<Clocks> = None;

// 295
impl Clocks {
    // 296
    pub(crate) fn init(cpu_clock_speed: CpuClock) {
        //critical_section::with(|_| {
        ESP_HAL_LOCK.lock(|| {
            unsafe { ACTIVE_CLOCKS = Some(Self::configure(cpu_clock_speed)) };
        })
    }

    // 302
    fn try_get<'a>() -> Option<&'a Clocks> {
        unsafe {
            // Safety: ACTIVE_CLOCKS is only set in `init` and never modified after that.
            let clocks = &*core::ptr::addr_of!(ACTIVE_CLOCKS);
            clocks.as_ref()
        }
    }

    /// Get the active clock configuration.
    // 311
    pub fn get<'a>() -> &'a Clocks {
        unwrap!(Self::try_get())
    }

    /// Returns the xtal frequency.
    ///
    /// This function will run the frequency estimation if called before
    /// [`crate::init()`].
    #[cfg(systimer)]
    #[inline]
    // 321
    pub(crate) fn xtal_freq() -> Rate {
        if esp_config::esp_config_str!("ESP_HAL_CONFIG_XTAL_FREQUENCY") == "auto"
            && let Some(clocks) = Self::try_get()
        {
            return clocks.xtal_clock;
        }

        Self::measure_xtal_frequency().frequency()
    }

    const fn xtal_frequency_from_config() -> Option<XtalClock> {
        let frequency_conf = esp_config::esp_config_str!("ESP_HAL_CONFIG_XTAL_FREQUENCY");

        for_each_soc_xtal_options!(
            (all $( ($freq:literal) ),*) => {
                paste::paste! {
                    return match frequency_conf.as_bytes() {
                        b"auto" => None,

                        // If the frequency is a pre-set value for the chip, return the associated enum variant.
                        $( _ if esp_config::esp_config_int_parse!(u32, frequency_conf) == $freq => Some(XtalClock::[<_ $freq M>]), )*

                        _ => None,
                    };
                }
            };
        );
    }

    fn measure_xtal_frequency() -> XtalClock {
        unwrap!(Self::xtal_frequency_from_config())
    }
}

#[cfg(esp32c3)]
// 438
impl Clocks {
    /// Configure the CPU clock speed.
    // 444
    pub(crate) fn configure(cpu_clock_speed: CpuClock) -> Self {
        let xtal_freq = Self::measure_xtal_frequency();

        let apb_freq;
        if cpu_clock_speed != CpuClock::default() {
            if cpu_clock_speed.mhz() <= xtal_freq.mhz() {
                apb_freq = ApbClock::ApbFreqOther(cpu_clock_speed.mhz());
                clocks_ll::esp32c3_rtc_update_to_xtal(xtal_freq, 1);
                clocks_ll::esp32c3_rtc_apb_freq_update(apb_freq);
            } else {
                let pll_freq = PllClock::Pll480MHz;
                apb_freq = ApbClock::ApbFreq80MHz;
                clocks_ll::esp32c3_rtc_bbpll_enable();
                clocks_ll::esp32c3_rtc_bbpll_configure(xtal_freq, pll_freq);
                clocks_ll::esp32c3_rtc_freq_to_pll_mhz(cpu_clock_speed);
                clocks_ll::esp32c3_rtc_apb_freq_update(apb_freq);
            }
        } else {
            apb_freq = ApbClock::ApbFreq80MHz;
        }

        Self {
            cpu_clock: cpu_clock_speed.frequency(),
            apb_clock: apb_freq.frequency(),
            xtal_clock: xtal_freq.frequency(),
        }
    }
}

#[cfg(any(bt, ieee802154, wifi))]
/// Tracks the number of references to the PHY clock.
static PHY_CLOCK_REF_COUNTER: embassy_sync::blocking_mutex::Mutex<RawMutex, Cell<u8>> =
    embassy_sync::blocking_mutex::Mutex::new(Cell::new(0));

#[cfg(any(bt, ieee802154, wifi))]
fn increase_phy_clock_ref_count_internal() {
    PHY_CLOCK_REF_COUNTER.lock(|phy_clock_ref_counter| {
        let phy_clock_ref_count = phy_clock_ref_counter.get();

        if phy_clock_ref_count == 0 {
            clocks_ll::enable_phy(true);
        }
        let new_phy_clock_ref_count = unwrap!(
            phy_clock_ref_count.checked_add(1),
            "PHY clock ref count overflowed."
        );

        phy_clock_ref_counter.set(new_phy_clock_ref_count);
    })
}

#[cfg(any(bt, ieee802154, wifi))]
fn decrease_phy_clock_ref_count_internal() {
    PHY_CLOCK_REF_COUNTER.lock(|phy_clock_ref_counter| {
        let new_phy_clock_ref_count = unwrap!(
            phy_clock_ref_counter.get().checked_sub(1),
            "PHY clock ref count underflowed. Either you forgot a PhyClockGuard, or used ModemClockController::decrease_phy_clock_ref_count incorrectly."
        );

        if new_phy_clock_ref_count == 0 {
            clocks_ll::enable_phy(false);
        }

        phy_clock_ref_counter.set(new_phy_clock_ref_count);
    })
}

#[inline]
/// Do any common initial initialization needed for the radio clocks
pub fn init_radio_clocks() {
    clocks_ll::init_clocks();
}

#[cfg(any(bt, ieee802154, wifi))]
#[derive(Debug)]
/// Prevents the PHY clock from being disabled.
///
/// As long as at least one [PhyClockGuard] exists, the PHY clock will remain
/// active. To release this guard, you can either let it go out of scope or use
/// [PhyClockGuard::release] to explicitly release it.
pub struct PhyClockGuard<'d> {
    _phantom: PhantomData<&'d ()>,
}

#[cfg(any(bt, ieee802154, wifi))]
impl PhyClockGuard<'_> {
    #[inline]
    /// Release the clock guard.
    ///
    /// The PHY clock will be disabled, if this is the last clock guard.
    pub fn release(self) {}
}

#[cfg(any(bt, ieee802154, wifi))]
impl Drop for PhyClockGuard<'_> {
    fn drop(&mut self) {
        decrease_phy_clock_ref_count_internal();
    }
}

/*
/// Control the radio peripheral clocks
//#[cfg(any(/*bt,ieee802154,*/ wifi))]
// 597
pub struct RadioClockController<'d> {
    _rcc: crate::peripherals::RADIO_CLK<'d>,
}
*/

#[cfg(any(bt, ieee802154, wifi))]
/// This trait provides common clock functionality for all modem peripherals.
pub trait ModemClockController<'d> {
    /// Enable the modem clock for this controller.
    fn enable_modem_clock(&mut self, enable: bool);

    // Enable the PHY clock and acquire a [PhyClockGuard].
    ///
    /// The PHY clock will only be disabled, once all [PhyClockGuard]'s of all
    /// modems were dropped.
    fn enable_phy_clock(&self) -> PhyClockGuard<'d> {
        increase_phy_clock_ref_count_internal();
        PhyClockGuard {
            _phantom: PhantomData,
        }
    }

    /// Decreases the PHY clock reference count for this modem ignoring
    /// currently alive [PhyClockGuard]s.
    ///
    /// # Panics
    /// This function panics if the PHY clock is inactive. If the ref count is
    /// lower than the number of alive [PhyClockGuard]s, dropping a guard can
    /// now panic.
    fn decrease_phy_clock_ref_count(&self) {
        decrease_phy_clock_ref_count_internal();
    }
}

#[cfg(wifi)]
impl<'d> ModemClockController<'d> for WIFI<'d> {
    fn enable_modem_clock(&mut self, enable: bool) {
        clocks_ll::enable_wifi(enable);
    }
}

#[cfg(wifi)]
impl WIFI<'_> {
    /// Reset the Wi-Fi MAC.
    pub fn reset_wifi_mac(&mut self) {
        clocks_ll::reset_wifi_mac();
    }
}

/*
#[cfg(any(/*bt, ieee802154,*/ wifi))]
//#[instability::unstable]
// 602
impl<'d> RadioClockController<'d> {
    /// Create a new instance of the radio clock controller
    //#[instability::unstable]
    // 605
    pub fn new(rcc: crate::peripherals::RADIO_CLK<'d>) -> Self {
        Self { _rcc: rcc }
    }

    /*
    /// Enable the PHY clocks
    #[cfg(phy)]
    #[inline]
    // 613
    pub fn enable_phy(&mut self, enable: bool) {
        clocks_ll::enable_phy(enable);
    }

    /// Enable the WiFi clocks
    #[cfg(wifi)]
    #[inline]
    // 629
    pub fn enable_wifi(&mut self, enable: bool) {
        clocks_ll::enable_wifi(enable);
    }

    /// Reset the MAC
    #[inline]
    // 644
    pub fn reset_wifi_mac(&mut self) {
        clocks_ll::reset_wifi_mac();
    }

    /// Do any common initial initialization needed
    //#[instability::unstable]
    #[inline]
    // 651
    pub fn init_clocks(&mut self) {
        clocks_ll::init_clocks();
    }
    */
}
*/
