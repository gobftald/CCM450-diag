// 50
#![no_std]

// MUST be the first module
// 59
mod fmt;

// 61
#[cfg(feature = "esp-radio")]
mod esp_radio;
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
    system::Cpu,
    timer::{AnyTimer, OneShotTimer, any::Degrade},
};

// 97
pub(crate) use scheduler::SCHEDULER;

// 100
use crate::task::IdleFn;

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

/// Starts the scheduler, with a custom idle hook.
///
/// The current context will be converted into the main task, and will be pinned to the first core.
///
/// The idle hook will be called when no tasks are ready to run. The idle hook's context is not
/// preserved. If you need to execute a longer process to enter a low-power state, make sure to call
/// the relevant code in a critical section.
///
/// The `timer` argument is a timer source that is used by the scheduler to
/// schedule internal tasks. The timer source can be any of the following:
///
/// - A timg `Timer` instance
/// - A systimer `Alarm` instance
/// - An `AnyTimer` instance
/// - A `OneShotTimer` instance
///
/// The `int0` argument must be `SoftwareInterrupt<0>` which will be used to trigger context
/// switches.
///
/// For an example, see the [crate-level documentation][self].
// 271
pub fn start_with_idle_hook(
    timer: impl TimerSource,
    int0: SoftwareInterrupt<'static, 0>,
    idle_hook: IdleFn,
) {
    #[cfg(feature = "rtos-trace")]
    {
        rtos_trace::trace::name_marker(TraceEvents::YieldTask as u32, "yield task");
        rtos_trace::trace::name_marker(TraceEvents::RunSchedule as u32, "run scheduler");
        rtos_trace::trace::name_marker(TraceEvents::TimerTickHandler as u32, "timer tick handler");
        rtos_trace::trace::name_marker(
            TraceEvents::ProcessTimerQueue as u32,
            "process timer queue",
        );
        rtos_trace::trace::name_marker(
            TraceEvents::ProcessEmbassyTimerQueue as u32,
            "process embassy timer queue",
        );
        rtos_trace::trace::start();
    }

    trace!("Starting scheduler for the first core");
    assert_eq!(Cpu::current(), Cpu::ProCpu);

    SCHEDULER.with(move |scheduler| {})
}
