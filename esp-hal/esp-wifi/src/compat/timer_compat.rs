// 1
use alloc::boxed::Box;

// 3
use esp_hal::sync::Locked;

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
    // 17
    fn new(f: unsafe extern "C" fn(*mut c_types::c_void), args: *mut c_types::c_void) -> Self {
        Self { f, args }
    }

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

// 45
impl Timer {
    // 46
    pub(crate) fn id(&self) -> usize {
        self.ets_timer as usize
    }
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

    // 60
    fn find(&mut self, ets_timer: *mut ets_timer) -> Option<&mut Box<Timer>> {
        let mut current = self.head.as_mut();
        while let Some(timer) = current {
            if core::ptr::eq(timer.ets_timer, ets_timer) {
                return Some(timer);
            }
            current = timer.next.as_mut();
        }

        None
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

    fn push(&mut self, to_add: Box<Timer>) -> Result<(), ()> {
        if self.head.is_none() {
            self.head = Some(to_add);
            return Ok(());
        }

        let mut current = self.head.as_mut();
        while let Some(timer) = current {
            if timer.next.is_none() {
                timer.next = Some(to_add);
                break;
            }
            current = timer.next.as_mut();
        }
        Ok(())
    }
}

// 145
pub(crate) static mut TIMERS: Locked<TimerQueue> = Locked::new(TimerQueue::new());

// 172
pub fn compat_timer_disarm(ets_timer: *mut ets_timer) {
    trace!("timer disarm");
    unsafe {
        TIMERS.with(|timers| {
            if let Some(timer) = timers.find(ets_timer) {
                trace!("timer_disarm {:x}", timer.id());
                timer.active = false;
            } else {
                trace!("timer_disarm {:x} not found", ets_timer as usize);
            }
        });
    }
}

// 203
pub(crate) fn compat_timer_setfn(
    ets_timer: *mut ets_timer,
    pfunction: unsafe extern "C" fn(*mut c_types::c_void),
    parg: *mut c_types::c_void,
) {
    trace!(
        "timer_setfn {:x} {:?} {:?}",
        ets_timer as usize, pfunction, parg
    );
    unsafe {
        let set = TIMERS.with(|timers| {
            if let Some(timer) = timers.find(ets_timer) {
                timer.callback = TimerCallback::new(pfunction, parg);
                timer.active = false;

                (*ets_timer).expire = 0;

                true
            } else {
                (*ets_timer).next = core::ptr::null_mut();
                (*ets_timer).period = 0;
                (*ets_timer).func = None;
                (*ets_timer).priv_ = core::ptr::null_mut();

                let timer =
                    crate::compat::malloc::calloc(1, core::mem::size_of::<Timer>()) as *mut Timer;
                (*timer).next = None;
                (*timer).ets_timer = ets_timer;
                (*timer).started = 0;
                (*timer).timeout = 0;
                (*timer).active = false;
                (*timer).periodic = false;
                (*timer).callback = TimerCallback::new(pfunction, parg);

                timers.push(Box::from_raw(timer)).is_ok()
            }
        });
    }
}
