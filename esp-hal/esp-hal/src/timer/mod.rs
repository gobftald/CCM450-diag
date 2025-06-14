//! # General-purpose Timers

// 50
use crate::{interrupt::InterruptHandler, time::Duration};

#[cfg(systimer)]
// 62
pub mod systimer;
#[cfg(any(timg0, /*timg1*/))]
// 64
pub mod timg;

/// Timer errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 69
pub enum Error {
    /// The timer is already active.
    TimerActive,
    /// The timer is not currently active.
    TimerInactive,
    /// The alarm is not currently active.
    AlarmInactive,
    /// The provided timeout is too large.
    InvalidTimeout,
}

/// Functionality provided by any timer peripheral.
// 81
pub trait Timer {
    /// Start the timer.
    // 84
    fn start(&self);

    /// Stop the timer.
    // 87
    fn stop(&self);

    /// Reset the timer value to 0.
    fn reset(&self);

    /// Is the timer running?
    // 95
    fn is_running(&self) -> bool;

    /// Load a target value into the timer.
    // 104
    fn load_value(&self, value: Duration) -> Result<(), Error>;

    /// Enable auto reload of the loaded value.
    // 108
    fn enable_auto_reload(&self, auto_reload: bool);

    /// Enable or disable the timer's interrupt.
    // 112
    fn enable_interrupt(&self, state: bool);

    /// Clear the timer's interrupt.
    // 115
    fn clear_interrupt(&self);

    /// Configures the interrupt handler.
    // 129
    fn set_interrupt_handler(&self, handler: InterruptHandler);
}

/// A one-shot timer.
//pub struct OneShotTimer<'d, Dm: DriverMode> {
// 136
pub struct OneShotTimer<'d> {
    inner: AnyTimer<'d>,
    //_ph: PhantomData<Dm>,
}

// 141
//impl<'d> OneShotTimer<'d, Blocking> {
impl<'d> OneShotTimer<'d> {
    /// Construct a new instance of [`OneShotTimer`].
    //pub fn new(inner: impl Timer + Into<AnyTimer<'d>>) -> OneShotTimer<'d, Blocking> {
    // 143
    pub fn new(inner: impl Timer + Into<AnyTimer<'d>>) -> OneShotTimer<'d> {
        Self {
            inner: inner.into(),
            //_ph: PhantomData,
        }
    }

    // came from 'impl OneShotTimer'
    /// Start counting until the given timeout and raise an interrupt
    // 271
    pub fn schedule(&mut self, timeout: Duration) -> Result<(), Error> {
        if self.inner.is_running() {
            self.inner.stop();
        }

        self.inner.clear_interrupt();
        self.inner.reset();

        self.inner.enable_auto_reload(false);
        self.inner.load_value(timeout)?;
        self.inner.start();

        Ok(())
    }

    /// Stop the timer
    //  287
    pub fn stop(&mut self) {
        self.inner.stop();
    }

    /// Set the interrupt handler
    ///
    /// Note that this will replace any previously set interrupt handler
    // 295
    pub fn set_interrupt_handler(&mut self, handler: InterruptHandler) {
        self.inner.set_interrupt_handler(handler);
    }

    /// Enable listening for interrupts
    // 300
    pub fn enable_interrupt(&mut self, enable: bool) {
        self.inner.enable_interrupt(enable);
    }

    /// Clear the interrupt flag
    // 305
    pub fn clear_interrupt(&mut self) {
        self.inner.clear_interrupt();
    }
}

// 416
crate::any_peripheral! {
    /// Any Timer peripheral.
    pub peripheral AnyTimer<'d> {
        //TimgTimer(timg::Timer<'d>),
        #[cfg(systimer)]
        SystimerAlarm(systimer::Alarm<'d>),
    }
}

// 425
impl Timer for AnyTimer<'_> {
    delegate::delegate! {
        to match &self.0 {
            //AnyTimerInner::TimgTimer(inner) => inner,
            #[cfg(systimer)]
            AnyTimerInner::SystimerAlarm(inner) => inner,
        } {
            fn start(&self);
            fn stop(&self);
            fn reset(&self);
            fn is_running(&self) -> bool;
            fn load_value(&self, value: Duration) -> Result<(), Error>;
            fn enable_auto_reload(&self, auto_reload: bool);
            fn clear_interrupt(&self);
            fn enable_interrupt(&self, state: bool);
            fn set_interrupt_handler(&self, handler: InterruptHandler);
        }
    }
}
