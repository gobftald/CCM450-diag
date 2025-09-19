// 1
use esp_hal::{
    interrupt::{InterruptHandler, Priority},
    sync::Locked,
    time::Rate,
    trapframe::TrapFrame,
};

#[cfg_attr(riscv, path = "riscv.rs")]
// 10
mod arch_specific;

// 12
pub(crate) use arch_specific::*;

// 14
use crate::TimeBase;

/// The timer responsible for time slicing.
// 17
const TIMESLICE_FREQUENCY: Rate = Rate::from_hz(crate::CONFIG.tick_rate_hz);

// 19
pub(crate) static TIMER: Locked<Option<TimeBase>> = Locked::new(None);

// 21
pub(crate) fn setup_timebase(mut timer: TimeBase) {
    // The timer needs to tick at Priority 1 to prevent accidentally interrupting
    // priority 1 limited locks.
    let cb: extern "C" fn() = unsafe { core::mem::transmute(timer_tick_handler as *const ()) };
    let handler = InterruptHandler::new(cb, Priority::Priority1);

    timer.set_interrupt_handler(handler);
    unwrap!(timer.start(TIMESLICE_FREQUENCY.as_duration()));
    TIMER.with(|t| {
        timer.enable_interrupt(true);
        t.replace(timer);
    });
}

// 35
pub(crate) fn clear_timer_interrupt() {
    TIMER.with(|timer| {
        unwrap!(timer.as_mut()).clear_interrupt();
    });
}

// 49
extern "C" fn timer_tick_handler(_context: &mut TrapFrame) {
    clear_timer_interrupt();

    // `task_switch` must be called on a single interrupt priority level only.
    // Because on ESP32 the software interrupt is triggered at priority 3 but
    // the timer interrupt is triggered at priority 1, we need to trigger the
    // software interrupt manually.
    cfg_if::cfg_if! {
        if #[cfg(esp32)] {
            yield_task();
        } else {
            super::task_switch(_context);
        }
    }
}
