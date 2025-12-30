// 50
#![no_std]

// 63
mod run_queue;
mod scheduler;
pub mod semaphore;
mod task;
mod timer;
mod wait_queue;

// 80
#[cfg(systimer)]
use esp_hal::timer::systimer::Alarm;
#[cfg(timergroup)]
use esp_hal::timer::timg::Timer;
use esp_hal::{
    Blocking,
    interrupt::software::SoftwareInterrupt,
    timer::{AnyTimer, OneShotTimer, any::Degrade},
};

// 105
type TimeBase = OneShotTimer<'static, Blocking>;

/// Timers that can be used as time drivers.
///
/// This trait is meant to be used only for the [`start`] function.
// 204
pub trait TimerSource: private::Sealed + 'static {
    /// Returns the timer source.
    fn timer(self) -> TimeBase;
}

// 209
mod private {
    pub trait Sealed {}
}

// 213
impl private::Sealed for TimeBase {}
impl private::Sealed for AnyTimer<'static> {}
#[cfg(timergroup)]
impl private::Sealed for Timer<'static> {}
#[cfg(systimer)]
impl private::Sealed for Alarm<'static> {}

// 220
impl TimerSource for TimeBase {
    fn timer(self) -> TimeBase {
        self
    }
}

// 226
impl TimerSource for AnyTimer<'static> {
    fn timer(self) -> TimeBase {
        TimeBase::new(self)
    }
}

// 232
#[cfg(timergroup)]
impl TimerSource for Timer<'static> {
    fn timer(self) -> TimeBase {
        TimeBase::new(self.degrade())
    }
}

// 239
#[cfg(systimer)]
impl TimerSource for Alarm<'static> {
    fn timer(self) -> TimeBase {
        TimeBase::new(self.degrade())
    }
}

/// Starts the scheduler.
///
/// The current context will be converted into the main task, and will be pinned to the first core.
///
/// This function is equivalent to [`start_with_idle_hook`], with the default idle hook. The default
/// idle hook will wait for an interrupt.
///
/// For information about the arguments, see [`start_with_idle_hook`].
// 254
pub fn start(timer: impl TimerSource, int0: SoftwareInterrupt<'static, 0>) {
    start_with_idle_hook(timer, int0, crate::task::idle_hook)
}
