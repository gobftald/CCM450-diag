use alloc::boxed::Box;

// 5
use crate::binary::{c_types, include::ets_timer};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 11
pub(crate) struct TimerCallback {
    f: unsafe extern "C" fn(*mut c_types::c_void),
    args: *mut c_types::c_void,
}

// 16
impl TimerCallback {
    // 21
    pub(crate) fn call(self) {
        unsafe { (self.f)(self.args) };
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
//34
pub(crate) struct Timer {
    pub ets_timer: *mut ets_timer,
    pub started: u64,
    pub timeout: u64,
    pub active: bool,
    pub periodic: bool,
    pub callback: TimerCallback,

    next: Option<Box<Timer>>,
}

// 51
pub(crate) struct TimerQueue {
    head: Option<Box<Timer>>,
}

// 55
impl TimerQueue {
    // 56
    const fn new() -> Self {
        Self { head: None }
    }

    // 72
    pub(crate) unsafe fn find_next_due(
        &mut self,
        current_timestamp: u64,
    ) -> Option<&mut Box<Timer>> {
        let mut current = self.head.as_mut();
        while let Some(timer) = current {
            if timer.active
                && crate::time::time_diff(timer.started, current_timestamp) >= timer.timeout
            {
                return Some(timer);
            }
            current = timer.next.as_mut();
        }

        None
    }
}

// 145
//pub(crate) static TIMERS: Locked<TimerQueue> = Locked::new(TimerQueue::new());
pub(crate) static mut TIMERS: TimerQueue = TimerQueue::new();
