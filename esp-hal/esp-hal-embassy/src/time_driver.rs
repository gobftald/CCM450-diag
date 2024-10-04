use core::cell::Cell;

use critical_section::Mutex;
use embassy_time_driver::{AlarmHandle, Driver};

pub const MAX_SUPPORTED_ALARM_COUNT: usize = 7;

// 19
struct AlarmState {
    pub callback: Cell<Option<(fn(*mut ()), *mut ())>>,
    pub allocated: Cell<bool>,
}

// 24
unsafe impl Send for AlarmState {}

// 26
impl AlarmState {
    pub const fn new() -> Self {
        Self {
            callback: Cell::new(None),
            allocated: Cell::new(false),
        }
    }
}

// 35
pub(super) struct EmbassyTimer {
    alarms: Mutex<[AlarmState; MAX_SUPPORTED_ALARM_COUNT]>,
}

// 39
embassy_time_driver::time_driver_impl!(static DRIVER: EmbassyTimer = EmbassyTimer {
    alarms: Mutex::new([const { AlarmState::new() }; MAX_SUPPORTED_ALARM_COUNT]),
});

// 123
impl Driver for EmbassyTimer {
    // 124
    fn now(&self) -> u64 {
        0
    }

    // 128
    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        critical_section::with(|cs| {
            for (i, alarm) in self.alarms.borrow(cs).iter().enumerate() {
                if !alarm.allocated.get() {
                    // set alarm so it is not overwritten
                    alarm.allocated.set(true);
                    return Some(AlarmHandle::new(i as u8));
                }
            }
            None
        })
    }

    // 141
    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        let n = alarm.id() as usize;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            // SyncExecutor as *mut ()
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    // 149
    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        true
    }
}
