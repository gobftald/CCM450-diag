//! # System Timer (SYSTIMER)
//!
//! ## Overview
//! The System Timer is a
#![cfg_attr(esp32s2, doc = "64-bit")]
#![cfg_attr(not(esp32s2), doc = "52-bit")]
//! timer which can be used, for example, to generate tick interrupts for an
//! operating system, or simply as a general-purpose timer.
//!
//! ## Configuration
//!
//! The timer consists of two counters, `Unit0` and `Unit1`. The counter values
//! can be monitored by 3 [`Alarm`]s
//!
//! It is recommended to pass the [`Alarm`]s into a high level driver like
//! [`OneShotTimer`](super::OneShotTimer) and
//! [`PeriodicTimer`](super::PeriodicTimer). Using the System timer directly is
//! only possible through the low level [`Timer`](crate::timer::Timer) trait.

// 20
use core::marker::PhantomData;

// 22
use super::Error;
use crate::{
    interrupt::{self, InterruptHandler},
    peripherals::{Interrupt, SYSTIMER},
    system::{Peripheral as PeripheralEnable, PeripheralClockControl},
    time::Duration,
};

/// System Timer driver.
// 46
pub struct SystemTimer<'d> {
    /// Alarm 0.
    pub alarm0: Alarm<'d>,
    /*
    /// Alarm 1.
    pub alarm1: Alarm<'d>,

    /// Alarm 2.
    pub alarm2: Alarm<'d>,
    */
}

// 57
impl<'d> SystemTimer<'d> {
    cfg_if::cfg_if! {
        if #[cfg(esp32s2)] {
            /// Bitmask to be applied to the raw register value.
            pub const BIT_MASK: u64 = u64::MAX;
            // Bitmask to be applied to the raw period register value.
            const PERIOD_MASK: u64 = 0x1FFF_FFFF;
        } else {
            /// Bitmask to be applied to the raw register value.
            pub const BIT_MASK: u64 = 0xF_FFFF_FFFF_FFFF;
            // Bitmask to be applied to the raw period register value.
            const PERIOD_MASK: u64 = 0x3FF_FFFF;
        }
    }

    /// Returns the tick frequency of the underlying timer unit.
    #[inline]
    // 74
    pub fn ticks_per_second() -> u64 {
        #[cfg(esp32c3)]
        // The counters and comparators are driven using `XTAL_CLK` (40 MHz)
        // The average clock frequency is fXTAL_CLK/2.5, which is 16 MHz.
        // The timer counting is incremented by 1/16 μs on each `CNT_CLK` cycle.
        const MULTIPLIER: u32 = 4;
        const DIVIDER: u32 = 10;

        let xtal_freq_mhz = crate::clock::Clocks::xtal_freq().as_hz();
        ((xtal_freq_mhz * MULTIPLIER) / DIVIDER) as u64
    }

    /// Create a new instance.
    // 98
    pub fn new(_systimer: SYSTIMER<'d>) -> Self {
        // Don't reset Systimer as it will break `time::Instant::now`, only enable it
        PeripheralClockControl::enable(PeripheralEnable::Systimer);

        Self {
            alarm0: Alarm::new(0),
            //alarm1: Alarm::new(1),
            //alarm2: Alarm::new(2),
        }
    }

    /// Get the current count of the given unit in the System Timer.
    // 113
    pub fn unit_value(unit: Unit) -> u64 {
        // This should be safe to access from multiple contexts
        // worst case scenario the second accessor ends up reading
        // an older time stamp

        unit.read_count()
    }
}

// 151
/// A
#[cfg_attr(esp32s2, doc = "64-bit")]
#[cfg_attr(not(esp32s2), doc = "52-bit")]
/// counter.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 157
pub enum Unit {
    /// Unit 0
    Unit0 = 0,
    #[cfg(not(esp32s2))]
    /// Unit 1
    Unit1 = 1,
}

// 165
impl Unit {
    // 166
    #[inline]
    fn channel(&self) -> u8 {
        *self as _
    }

    // 237
    fn read_count(&self) -> u64 {
        // This can be a shared reference as long as this type isn't Sync.

        let channel = self.channel() as usize;
        let systimer = SYSTIMER::regs();

        systimer.unit_op(channel).write(|w| w.update().set_bit());
        while !systimer.unit_op(channel).read().value_valid().bit_is_set() {}

        // Read LO, HI, then LO again, check that LO returns the same value.
        // This accounts for the case when an interrupt may happen between reading
        // HI and LO values (or the other core updates the counter mid-read), and this
        // function may get called from the ISR. In this case, the repeated read
        // will return consistent values.
        let unit_value = systimer.unit_value(channel);
        let mut lo_prev = unit_value.lo().read().bits();
        loop {
            let lo = lo_prev;
            let hi = unit_value.hi().read().bits();
            lo_prev = unit_value.lo().read().bits();

            if lo == lo_prev {
                return ((hi as u64) << 32) | lo as u64;
            }
        }
    }
}

/// An alarm unit
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 268
pub struct Alarm<'d> {
    comp: u8,
    unit: Unit,
    _lifetime: PhantomData<&'d mut ()>,
}

// 274
impl Alarm<'_> {
    // 275
    const fn new(comp: u8) -> Self {
        Alarm {
            comp,
            unit: Unit::Unit0,
            _lifetime: PhantomData,
        }
    }

    /// Unsafely clone this peripheral reference.
    ///
    /// # Safety
    ///
    /// You must ensure that you're only using one instance of this type at a
    /// time.
    // 289
    pub unsafe fn clone_unchecked(&self) -> Self {
        Self {
            comp: self.comp,
            unit: self.unit,
            _lifetime: PhantomData,
        }
    }

    /// Returns the comparator's number.
    #[inline]
    // 307
    fn channel(&self) -> u8 {
        self.comp
    }

    /// Enables/disables the comparator. If enabled, this means
    /// it will generate interrupt based on its configuration.
    // 313
    fn set_enable(&self, enable: bool) {
        //lock(&CONF_LOCK, || {
        #[cfg(not(esp32s2))]
        SYSTIMER::regs().conf().modify(|_, w| match self.channel() {
            0 => w.target0_work_en().bit(enable),
            //1 => w.target1_work_en().bit(enable),
            //2 => w.target2_work_en().bit(enable),
            _ => unreachable!(),
        });
        //});

        // Note: The ESP32-S2 doesn't require a lock because each
        // comparator's enable bit in a different register.
        #[cfg(esp32s2)]
        SYSTIMER::regs()
            .target_conf(self.channel() as usize)
            .modify(|_r, w| w.work_en().bit(enable));
    }

    /// Returns true if the comparator has been enabled. This means
    /// it will generate interrupt based on its configuration.
    // 334
    fn is_enabled(&self) -> bool {
        #[cfg(not(esp32s2))]
        match self.channel() {
            0 => SYSTIMER::regs().conf().read().target0_work_en().bit(),
            //1 => SYSTIMER::regs().conf().read().target1_work_en().bit(),
            //2 => SYSTIMER::regs().conf().read().target2_work_en().bit(),
            _ => unreachable!(),
        }

        #[cfg(esp32s2)]
        SYSTIMER::regs()
            .target_conf(self.channel() as usize)
            .read()
            .work_en()
            .bit()
    }

    /// Set the mode of the comparator to be either target or periodic.
    // 360
    fn set_mode(&self, mode: ComparatorMode) {
        let is_period_mode = match mode {
            ComparatorMode::Period => true,
            ComparatorMode::Target => false,
        };
        SYSTIMER::regs()
            .target_conf(self.channel() as usize)
            .modify(|_, w| w.period_mode().bit(is_period_mode));
    }

    /// Get the current mode of the comparator, which is either target or
    /// periodic.
    // 372
    fn mode(&self) -> ComparatorMode {
        if SYSTIMER::regs()
            .target_conf(self.channel() as usize)
            .read()
            .period_mode()
            .bit()
        {
            ComparatorMode::Period
        } else {
            ComparatorMode::Target
        }
    }

    /// Set how often the comparator should generate an interrupt when in
    /// periodic mode.
    // 387
    fn set_period(&self, value: u32) {
        let systimer = SYSTIMER::regs();
        let tconf = systimer.target_conf(self.channel() as usize);
        unsafe { tconf.modify(|_, w| w.period().bits(value)) };
        #[cfg(not(esp32s2))]
        {
            let comp_load = systimer.comp_load(self.channel() as usize);
            comp_load.write(|w| w.load().set_bit());
        }
    }

    // 399
    fn set_target(&self, value: u64) {
        let systimer = SYSTIMER::regs();
        let target = systimer.trgt(self.channel() as usize);
        target.hi().write(|w| w.hi().set((value >> 32) as u32));
        target
            .lo()
            .write(|w| w.lo().set((value & 0xFFFF_FFFF) as u32));
        #[cfg(not(esp32s2))]
        {
            let comp_load = systimer.comp_load(self.channel() as usize);
            comp_load.write(|w| w.load().set_bit());
        }
    }

    /// Set the interrupt handler for this comparator.
    // 414
    fn set_interrupt_handler(&self, handler: InterruptHandler) {
        let interrupt = match self.channel() {
            0 => Interrupt::SYSTIMER_TARGET0,
            //1 => Interrupt::SYSTIMER_TARGET1,
            //2 => Interrupt::SYSTIMER_TARGET2,
            _ => unreachable!(),
        };

        //for core in crate::system::Cpu::other() {
        //crate::interrupt::disable(core, interrupt);
        crate::interrupt::disable(interrupt as u8);
        //}

        #[cfg(not(esp32s2))]
        unsafe {
            interrupt::bind_interrupt(interrupt, handler.handler());
        }

        unwrap!(interrupt::enable(interrupt, handler.priority()));
    }
}

/// The modes of a comparator.
#[derive(Copy, Clone)]
// 468
enum ComparatorMode {
    /// The comparator will generate interrupts periodically.
    Period,

    /// The comparator will generate an interrupt when the unit reaches the
    /// target.
    Target,
}

// 477
impl super::Timer for Alarm<'_> {
    // 478
    fn start(&self) {
        self.set_enable(true);
    }

    // 482
    fn stop(&self) {
        self.set_enable(false);
    }

    // 486
    fn reset(&self) {
        #[cfg(esp32s2)]
        // Run at XTAL freq, not 80 * XTAL freq:
        SYSTIMER::regs()
            .step()
            .modify(|_, w| unsafe { w.xtal_step().bits(0x1) });

        #[cfg(not(esp32s2))]
        SYSTIMER::regs()
            .conf()
            // counter is always running, even when cpu in 'wfi' mode
            .modify(|_, w| w.timer_unit0_core0_stall_en().clear_bit());
    }

    // 499
    fn is_running(&self) -> bool {
        self.is_enabled()
    }

    // 514
    fn load_value(&self, value: Duration) -> Result<(), Error> {
        let mode = self.mode();

        let us = value.as_micros();
        let ticks = us * (SystemTimer::ticks_per_second() / 1_000_000);

        if matches!(mode, ComparatorMode::Period) {
            // Period mode

            // The `SYSTIMER_TARGETx_PERIOD` field is 26-bits wide (or
            // 29-bits on the ESP32-S2), so we must ensure that the provided
            // value is not too wide:
            if (ticks & !SystemTimer::PERIOD_MASK) != 0 {
                return Err(Error::InvalidTimeout);
            }

            self.set_period(ticks as u32);

            // Clear and then set SYSTIMER_TARGETx_PERIOD_MODE to configure COMPx into
            // period mode
            self.set_mode(ComparatorMode::Target);
            self.set_mode(ComparatorMode::Period);
        } else {
            // Target mode

            // The counters/comparators are 52-bits wide (except on ESP32-S2,
            // which is 64-bits), so we must ensure that the provided value
            // is not too wide:
            #[cfg(not(esp32s2))]
            if (ticks & !SystemTimer::BIT_MASK) != 0 {
                return Err(Error::InvalidTimeout);
            }

            let v = self.unit.read_count();
            let t = v + ticks;

            self.set_target(t);
        }

        Ok(())
    }

    // 556
    fn enable_auto_reload(&self, auto_reload: bool) {
        // If `auto_reload` is true use Period Mode, otherwise use Target Mode:
        let mode = if auto_reload {
            ComparatorMode::Period
        } else {
            ComparatorMode::Target
        };
        self.set_mode(mode)
    }

    // 566
    fn enable_interrupt(&self, state: bool) {
        //lock(&INT_ENA_LOCK, || {
        SYSTIMER::regs()
            .int_ena()
            .modify(|_, w| w.target(self.channel()).bit(state));
        //});
    }

    // 574
    fn clear_interrupt(&self) {
        SYSTIMER::regs()
            .int_clr()
            .write(|w| w.target(self.channel()).clear_bit_by_one());
    }

    // 606
    fn set_interrupt_handler(&self, handler: InterruptHandler) {
        self.set_interrupt_handler(handler)
    }
}
