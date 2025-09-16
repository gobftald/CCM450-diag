//! # Real-Time Control and Low-power Management (RTC_CNTL)
//!
//! ## Overview
//!
//! The RTC_CNTL peripheral is responsible for managing the low-power modes on
//! the chip.
//!
//! ## Configuration
//!
//! It also includes the necessary configurations and constants for clock
//! sources and low-power management.

#[cfg_attr(esp32c3, path = "rtc/esp32c3.rs")]
// 145
pub(crate) mod rtc;

// 153
use crate::peripherals::LPWR as LP_WDT;

/// Low-power Management
// 279
pub struct Rtc<'d> {
    _inner: crate::peripherals::LPWR<'d>,
    /// Reset Watchdog Timer.
    pub rwdt: Rwdt,
    #[cfg(any(esp32c2, esp32c3, esp32c6, esp32h2, esp32s3))]
    /// Super Watchdog
    pub swd: Swd,
}

// 288
impl<'d> Rtc<'d> {
    /// Create a new instance in [crate::Blocking] mode.
    ///
    /// Optionally an interrupt handler can be bound.
    // 292
    pub fn new(rtc_cntl: crate::peripherals::LPWR<'d>) -> Self {
        rtc::init();
        // we don't plan to use low power mode
        //rtc::configure_clock();

        let this = Self {
            _inner: rtc_cntl,
            rwdt: Rwdt::new(),
            #[cfg(any(esp32c2, esp32c3, esp32c6, esp32h2, esp32s3))]
            swd: Swd::new(),
        };

        #[cfg(any(esp32, esp32s2, esp32s3, esp32c3, esp32c6, esp32c2))]
        // we don't plane to use 'deep sleep' mode yet
        // RtcSleepConfig::base_settings(&this);
        //
        this
    }
}

/// RTC Watchdog Timer.
// 913
pub struct Rwdt;

/// RTC Watchdog Timer driver.
// 922
impl Rwdt {
    /// Create a new RTC watchdog timer instance
    // 924
    pub fn new() -> Self {
        Self
    }

    // 936
    /// Disable the watchdog timer instance.
    pub fn disable(&mut self) {
        self.set_enabled(false);
    }

    // 999
    fn set_write_protection(&mut self, enable: bool) {
        let rtc_cntl = LP_WDT::regs();

        let wkey = if enable { 0u32 } else { 0x50D8_3AA1 };

        rtc_cntl.wdtwprotect().write(|w| unsafe { w.bits(wkey) });
    }

    // 1007
    fn set_enabled(&mut self, enable: bool) {
        let rtc_cntl = LP_WDT::regs();

        self.set_write_protection(false);

        if !enable {
            rtc_cntl.wdtconfig0().modify(|_, w| unsafe { w.bits(0) });
        }

        self.set_write_protection(true);
    }
}

#[cfg(any(esp32c2, esp32c3, esp32c6, esp32h2, esp32s3))]
/// Super Watchdog
// 1142
pub struct Swd;

#[cfg(any(esp32c2, esp32c3, esp32c6, esp32h2, esp32s3))]
/// Super Watchdog driver
// 1146
impl Swd {
    /// Create a new super watchdog timer instance
    // 1148
    pub fn new() -> Self {
        Self
    }

    // 1158
    /// Disable the watchdog timer instance
    pub fn disable(&mut self) {
        self.set_enabled(false);
    }

    // 1163
    /// Enable/disable write protection for WDT registers
    fn set_write_protection(&mut self, enable: bool) {
        let rtc_cntl = LP_WDT::regs();

        #[cfg(not(any(esp32c6, esp32h2)))]
        let wkey = if enable { 0u32 } else { 0x8F1D_312A };
        #[cfg(any(esp32c6, esp32h2))]
        let wkey = if enable { 0u32 } else { 0x50D8_3AA1 };

        rtc_cntl
            .swd_wprotect()
            .write(|w| unsafe { w.swd_wkey().bits(wkey) });
    }

    // 1176
    fn set_enabled(&mut self, enable: bool) {
        let rtc_cntl = LP_WDT::regs();

        self.set_write_protection(false);
        rtc_cntl
            .swd_conf()
            .write(|w| w.swd_auto_feed_en().bit(!enable));
        self.set_write_protection(true);
    }
}
