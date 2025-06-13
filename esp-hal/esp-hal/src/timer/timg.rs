//! # Timer Group (TIMG)
//!
//! ## Overview
//!
//! The Timer Group (TIMG) peripherals contain one or more general-purpose
//! timers, plus one or more watchdog timers.
//!
//! The general-purpose timers are based on a 16-bit pre-scaler and a 54-bit
//! auto-reload-capable up-down counter.
//!
//! ## Configuration
//!
//! The timers have configurable alarms, which are triggered when the internal
//! counter of the timers reaches a specific target value. The timers are
//! clocked using the APB clock source.
//!
//! Typically, a general-purpose timer can be used in scenarios such as:
//!
//! - Generate period alarms; trigger events periodically
//! - Generate one-shot alarms; trigger events once
//! - Free-running; fetching a high-resolution timestamp on demand

use core::marker::PhantomData;

// 76
use crate::{pac::timg0::RegisterBlock, peripherals::TIMG0, system::PeripheralClockControl};

// 103
pub struct TimerGroup<'d, T>
where
    T: TimerGroupInstance + 'd,
{
    _timer_group: PhantomData<T>,
    /// Timer 0
    pub timer0: Timer<'d>,
    /*
    /// Timer 1
    #[cfg(timg_timer1)]
    pub timer1: Timer<'d>,
    */
    /// Watchdog timer
    pub wdt: Wdt<T>,
}

// 118
pub trait TimerGroupInstance {
    fn id() -> u8;
    fn register_block() -> *const RegisterBlock;
    fn configure_src_clk();
    fn enable_peripheral();
    fn reset_peripheral();
    fn configure_wdt_src_clk();
}

// 128
impl TimerGroupInstance for TIMG0<'_> {
    // 129
    fn id() -> u8 {
        0
    }

    #[inline(always)]
    // 134
    fn register_block() -> *const RegisterBlock {
        Self::regs()
    }

    // 138
    fn configure_src_clk() {
        cfg_if::cfg_if! {
            if #[cfg(esp32)] {
                // ESP32 has only APB clock source, do nothing
            } else if #[cfg(any(esp32c2, esp32c3, esp32s2, esp32s3))] {
                unsafe {
                    (*<Self as TimerGroupInstance>::register_block())
                        .t(0)
                        .config()
                        .modify(|_, w| w.use_xtal().clear_bit());
                }
            } else if #[cfg(any(esp32c6, esp32h2))] {
                crate::peripherals::PCR::regs()
                    .timergroup0_timer_clk_conf()
                    .modify(|_, w| unsafe { w.tg0_timer_clk_sel().bits(TIMG_DEFAULT_CLK_SRC) });
            }
        }
    }

    // 157
    fn enable_peripheral() {
        PeripheralClockControl::enable(crate::system::Peripheral::Timg0);
    }

    // 161
    fn reset_peripheral() {
        // FIXME: for TIMG0 do nothing for now because the reset breaks
        // `time::Instant::now`
    }

    // 166
    fn configure_wdt_src_clk() {
        cfg_if::cfg_if! {
            if #[cfg(any(esp32, esp32s2, esp32s3))] {
                // ESP32, ESP32-S2, and ESP32-S3 use only ABP, do nothing
            } else if #[cfg(any(esp32c2, esp32c3))] {
                unsafe {
                    (*<Self as TimerGroupInstance>::register_block())
                        .wdtconfig0()
                        .modify(|_, w| w.wdt_use_xtal().clear_bit());
                }
            } else if #[cfg(any(esp32c6, esp32h2))] {
                crate::peripherals::PCR::regs()
                    .timergroup0_wdt_clk_conf()
                    .modify(|_, w| unsafe { w.tg0_wdt_clk_sel().bits(1) });
            }
        }
    }
}

// 246
impl<'d, T> TimerGroup<'d, T>
where
    T: TimerGroupInstance + 'd,
{
    /// Construct a new instance of [`TimerGroup`] in blocking mode
    // 251
    pub fn new(_timer_group: T) -> Self {
        T::reset_peripheral();
        T::enable_peripheral();

        T::configure_src_clk();

        Self {
            _timer_group: PhantomData,
            timer0: Timer {
                timer: 0,
                tg: T::id(),
                register_block: T::register_block(),
                _lifetime: PhantomData,
            },
            /*
            #[cfg(timg_timer1)]
            timer1: Timer {
                timer: 1,
                tg: T::id(),
                register_block: T::register_block(),
                _lifetime: PhantomData,
            },
            */
            wdt: Wdt::new(),
        }
    }
}

/// A timer within a Timer Group.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 363
pub struct Timer<'d> {
    register_block: *const RegisterBlock,
    _lifetime: PhantomData<&'d mut ()>,
    timer: u8,
    tg: u8,
}

/// Watchdog timer
// 623
pub struct Wdt<TG> {
    phantom: PhantomData<TG>,
}

/// Watchdog driver
// 628
impl<TG> Wdt<TG>
where
    TG: TimerGroupInstance,
{
    /// Construct a new instance of [`Wdt`]
    // 633
    pub fn new() -> Self {
        TG::configure_wdt_src_clk();

        Self {
            phantom: PhantomData,
        }
    }

    /// Disable the watchdog timer instance
    // 649
    pub fn disable(&mut self) {
        // SAFETY: The `TG` instance being modified is owned by `self`, which is behind
        //         a mutable reference.
        unsafe { self.set_wdt_enabled(false) };
    }

    /// Forcibly enable or disable the watchdog timer
    ///
    /// # Safety
    ///
    /// This bypasses the usual ownership rules for the peripheral, so users
    /// must take care to ensure that no driver instance is active for the
    /// timer.
    // 662
    pub unsafe fn set_wdt_enabled(&mut self, enabled: bool) {
        let reg_block = unsafe { &*TG::register_block() };

        self.set_write_protection(false);

        if !enabled {
            reg_block.wdtconfig0().write(|w| unsafe { w.bits(0) });
        } else {
            /*  we will implement this if wdt will be enabled
            reg_block.wdtconfig0().write(|w| w.wdt_en().bit(true));

            reg_block
                .wdtconfig0()
                .write(|w| w.wdt_flashboot_mod_en().bit(false));

            #[cfg_attr(esp32, allow(unused_unsafe))]
            reg_block.wdtconfig0().write(|w| unsafe {
                w.wdt_en()
                    .bit(true)
                    .wdt_stg0()
                    .bits(MwdtStageAction::ResetSystem as u8)
                    .wdt_cpu_reset_length()
                    .bits(7)
                    .wdt_sys_reset_length()
                    .bits(7)
                    .wdt_stg1()
                    .bits(MwdtStageAction::Off as u8)
                    .wdt_stg2()
                    .bits(MwdtStageAction::Off as u8)
                    .wdt_stg3()
                    .bits(MwdtStageAction::Off as u8)
            });

            #[cfg(any(esp32c2, esp32c3, esp32c6))]
            reg_block
                .wdtconfig0()
                .modify(|_, w| w.wdt_conf_update_en().set_bit());
            */
        }

        self.set_write_protection(true);
    }

    // 714
    fn set_write_protection(&mut self, enable: bool) {
        let reg_block = unsafe { &*TG::register_block() };

        let wkey = if enable { 0u32 } else { 0x50D8_3AA1u32 };

        reg_block
            .wdtwprotect()
            .write(|w| unsafe { w.wdt_wkey().bits(wkey) });
    }
}
