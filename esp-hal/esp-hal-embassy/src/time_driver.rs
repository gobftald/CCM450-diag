//! Embassy time driver implementation
//!
//! The time driver is responsible for keeping time, as well as to manage the
//! wait queue for embassy-time.

/// embassy requires us to implement the [embassy_time_driver::Driver] trait,
/// which we do here. This trait needs us to be able to tell the current time,
/// as well as to schedule a wake-up at a certain time.
///
/// We are free to choose how we implement these features, and we provide
/// three options:
///
/// - If the `generic` feature is enabled, we implement a single timer queue,
///   using the implementation provided by embassy-time-queue-driver.
/// - If the `single-integrated` feature is enabled, we implement a single timer
///   queue, using our own integrated timer implementation. Our implementation
///   is a copy of the embassy integrated timer queue, with the addition of
///   clearing the "owner" information upon dequeueing.
/// I've chosed this implementation but w/o 'sync and mutex' stuff.
/// So 'RawQueue' of 'TimerQueue' in timer_queue.rs is rally a copy of 'Queue'
/// in queue_integrated.rs in embassy-time-queue-utils crate.
///
/// - If the `multiple-integrated` feature is enabled, we provide a separate
///   timer queue for each executor. We store a separate timer queue for each
///   executor, and we use the scheduled task's owner to determine which queue
///   to use. This mode allows us to use less disruptive locks around the timer
///   queue, but requires more timers - one per timer queue.
// 9
use embassy_time_driver::Driver;
use esp_hal::time::Instant;

// 109
pub(super) struct EmbassyTimer {
    // /// The timer queue, if we use a single one (single-integrated, or generic).
    //#[cfg(single_queue)]
    //pub(crate) inner: crate::timer_queue::TimerQueue,
    //alarms: [Alarm; MAX_SUPPORTED_ALARM_COUNT],
    //available_timers: Locked<Option<&'static mut [Timer]>>,
}

// 139
embassy_time_driver::time_driver_impl!(static DRIVER: EmbassyTimer = EmbassyTimer {
    // Single queue, needs maximum priority.
    //#[cfg(single_queue)]
    //inner: crate::timer_queue::TimerQueue::new(Priority::max()),
    //alarms: alarms!(0, 1, 2, 3, 4, 5, 6),
    //available_timers: Locked::new(None),
});

// 313
impl Driver for EmbassyTimer {
    fn now(&self) -> u64 {
        Instant::now().duration_since_epoch().as_micros()
    }
}
