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

// 71
use super::Error;

// 76
use crate::{
    clock::Clocks,
    interrupt::{self, InterruptHandler},
    pac::timg0::RegisterBlock,
    peripherals::{Interrupt, TIMG0},
    system::PeripheralClockControl,
    time::{Duration, Rate},
};

/// A timer group consisting of
/// a general purpose timer
/// and a watchdog timer.
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
                        .modify(|_, w| w.use_xtal().clear_bit());   // use APB
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
                timer: TimerId::Timer0,
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

// 277
impl super::Timer for Timer<'_> {
    // 278
    fn start(&self) {
        self.set_counter_active(false);
        self.set_alarm_active(false);

        self.reset_counter();
        self.set_counter_decrementing(false);

        self.set_counter_active(true);
        self.set_alarm_active(true);
    }

    // 289
    fn stop(&self) {
        self.set_counter_active(false);
    }

    // 293
    fn reset(&self) {
        self.reset_counter()
    }

    // 297
    fn is_running(&self) -> bool {
        self.is_counter_active()
    }

    // 305
    fn load_value(&self, value: Duration) -> Result<(), Error> {
        self.load_value(value)
    }

    // 309
    fn enable_auto_reload(&self, auto_reload: bool) {
        self.set_auto_reload(auto_reload)
    }

    // 313
    fn enable_interrupt(&self, state: bool) {
        self.set_interrupt_enabled(state);
    }

    // 317
    fn clear_interrupt(&self) {
        self.clear_interrupt()
    }

    // 351
    fn set_interrupt_handler(&self, handler: InterruptHandler) {
        self.set_interrupt_handler(handler)
    }
}

/// A timer within a Timer Group.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 363
pub struct Timer<'d> {
    register_block: *const RegisterBlock,
    _lifetime: PhantomData<&'d mut ()>,
    timer: TimerId,
    tg: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum TimerId {
    Timer0,
    #[cfg(timergroup_timg_has_timer1)]
    Timer1,
}

/// Timer peripheral instance
// 374
impl Timer<'_> {
    /// Unsafely clone this peripheral reference.
    ///
    /// # Safety
    ///
    /// You must ensure that you're only using one instance of this type at a
    /// time.
    // 381
    pub unsafe fn clone_unchecked(&self) -> Self {
        Self {
            register_block: self.register_block,
            timer: self.timer,
            tg: self.tg,
            _lifetime: PhantomData,
        }
    }

    // 427
    pub(crate) fn set_interrupt_handler(&self, handler: InterruptHandler) {
        let interrupt = match (self.timer_group(), self.timer_number()) {
            (0, 0) => Interrupt::TG0_T0_LEVEL,
            /*
            #[cfg(timg_timer1)]
            (0, 1) => Interrupt::TG0_T1_LEVEL,
            #[cfg(timg1)]
            (1, 0) => Interrupt::TG1_T0_LEVEL,
            #[cfg(all(timg_timer1, timg1))]
            (1, 1) => Interrupt::TG1_T1_LEVEL,
            */
            _ => unreachable!(),
        };

        for core in crate::system::Cpu::other() {
            crate::interrupt::disable(core, interrupt);
        }
        unsafe { interrupt::bind_interrupt(interrupt, handler.handler()) };
        unwrap!(interrupt::enable(interrupt, handler.priority()));
    }

    // 446
    fn register_block(&self) -> &RegisterBlock {
        unsafe { &*self.register_block }
    }

    // 450
    fn timer_group(&self) -> u8 {
        self.tg
    }

    // 425
    fn timer_number(&self) -> u8 {
        self.timer as u8
    }

    // 429
    fn t(&self) -> &crate::pac::timg0::T {
        self.register_block().t(self.timer_number().into())
    }

    // 433
    fn reset_counter(&self) {
        let t = self.t();

        t.loadlo().write(|w| unsafe { w.load_lo().bits(0) });
        t.loadhi().write(|w| unsafe { w.load_hi().bits(0) });

        t.load().write(|w| unsafe { w.load().bits(1) });
    }

    // 442
    fn set_counter_active(&self, state: bool) {
        self.t().config().modify(|_, w| w.en().bit(state));
    }

    // 446
    fn is_counter_active(&self) -> bool {
        self.t().config().read().en().bit_is_set()
    }

    // 450
    fn set_counter_decrementing(&self, decrementing: bool) {
        self.t()
            .config()
            .modify(|_, w| w.increase().bit(!decrementing));
    }

    // 456
    fn set_auto_reload(&self, auto_reload: bool) {
        self.t()
            .config()
            .modify(|_, w| w.autoreload().bit(auto_reload));
    }

    // 462
    fn set_alarm_active(&self, state: bool) {
        self.t().config().modify(|_, w| w.alarm_en().bit(state));
    }

    // 466
    fn load_value(&self, value: Duration) -> Result<(), Error> {
        #[cfg(not(esp32h2))]
        let clk_src = Clocks::get().apb_clock;
        let Some(ticks) = timeout_to_ticks(value, clk_src, self.divider()) else {
            return Err(Error::InvalidTimeout);
        };

        // The counter is 54-bits wide, so we must ensure that the provided
        // value is not too wide:
        if (ticks & !0x3F_FFFF_FFFF_FFFF) != 0 {
            return Err(Error::InvalidTimeout);
        }

        let high = (ticks >> 32) as u32;
        let low = (ticks & 0xFFFF_FFFF) as u32;

        let t = self.t();

        t.alarmlo().write(|w| unsafe { w.alarm_lo().bits(low) });
        t.alarmhi().write(|w| unsafe { w.alarm_hi().bits(high) });

        Ok(())
    }

    // 496
    fn clear_interrupt(&self) {
        self.register_block()
            .int_clr()
            .write(|w| w.t(self.timer as _).clear_bit_by_one());
        let periodic = self.t().config().read().autoreload().bit_is_set();
        self.set_alarm_active(periodic);
    }

    // 529
    fn divider(&self) -> u32 {
        let t = self.t();

        // From the ESP32 TRM, "11.2.1 16­-bit Prescaler and Clock Selection":
        //
        // "The prescaler can divide the APB clock by a factor from 2 to 65536.
        // Specifically, when TIMGn_Tx_DIVIDER is either 1 or 2, the clock divisor is 2;
        // when TIMGn_Tx_DIVIDER is 0, the clock divisor is 65536. Any other value will
        // cause the clock to be divided by exactly that value."
        match t.config().read().divider().bits() {
            0 => 65536,
            1 | 2 => 2,
            n => n as u32,
        }
    }

    // 553
    fn set_interrupt_enabled(&self, state: bool) {
        cfg_if::cfg_if! {
            if #[cfg(any(esp32, esp32s2))] {
                // On ESP32 and S2, the `int_ena` register is ineffective - interrupts fire even
                // without int_ena enabling them. We use level interrupts so that we have a status
                // bit available.
                self.register_block()
                    .t(self.timer as usize)
                    .config()
                    .modify(|_, w| w.level_int_en().bit(state));
            /* we have no timg1 yet
            } else if #[cfg(timergroup_timg_has_timer1)] {
                lock(&INT_ENA_LOCK[self.timer_group() as usize], || {
                    self.register_block()
                        .int_ena()
                        .modify(|_, w| w.t(self.timer_number()).bit(state));
                });
            */
            } else {
                self.register_block()
                    .int_ena()
                    .modify(|_, w| w.t(0).bit(state));
            }
        }
    }
}

fn timeout_to_ticks(timeout: Duration, clock: Rate, divider: u32) -> Option<u64> {
    let micros = timeout.as_micros();
    let ticks_per_sec = (clock.as_hz() / divider) as u64;

    micros.checked_mul(ticks_per_sec).map(|n| n / 1_000_000)
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

impl<TG> Default for Wdt<TG>
where
    TG: TimerGroupInstance,
{
    fn default() -> Self {
        Self::new()
    }
}
