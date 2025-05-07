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

// 23
use crate::peripherals::SYSTIMER;

/// System Timer driver.
// 46
pub struct SystemTimer<'d> {
    /// Alarm 0.
    pub alarm0: Alarm<'d>,

    /// Alarm 1.
    pub alarm1: Alarm<'d>,

    /// Alarm 2.
    pub alarm2: Alarm<'d>,
}

// 47
impl<'d> SystemTimer<'d> {
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

fn handle_alarm(alarm: u8) {}
