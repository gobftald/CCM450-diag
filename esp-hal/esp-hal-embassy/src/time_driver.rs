//! Embassy time driver implementation
//!
//! The time driver is responsible for keeping time, as well as to manage the
//! wait queue for embassy-time.

// 9
use embassy_time_driver::Driver;
use esp_hal::{
    interrupt::{InterruptHandler, Priority},
    time::{Duration, Instant},
    timer::{Error, OneShotTimer},
};

// 18
//pub type Timer = OneShotTimer<'static, Blocking>;
pub type Timer = OneShotTimer<'static>;

/// Alarm handle, assigned by the driver.
#[derive(Clone, Copy)]
// 22
pub(crate) struct AlarmHandle {
    id: usize,
}

// 26
impl AlarmHandle {
    /// Create an AlarmHandle
    ///
    /// Safety: May only be called by the current global Driver impl.
    /// The impl is allowed to rely on the fact that all `AlarmHandle` instances
    /// are created by itself in unsafe code (e.g. indexing operations)
    // 32
    pub unsafe fn new(id: usize) -> Self {
        Self { id }
    }

    // 36
    pub fn update(&self, expiration: u64) -> bool {
        if expiration == u64::MAX {
            true
        } else {
            unsafe { DRIVER.set_alarm(*self, expiration) }
        }
    }
}

// 46
//enum AlarmState {
enum Alarm {
    Created(extern "C" fn()),
    // Initialized(&'static mut Timer), // but it is  a 'pub type Timer = OneShotTimer<'static>;'
    // and we want to use Timer trate
    Initialized(&'static mut Timer),
}

// 49
//impl AlarmState {
impl Alarm {
    //fn initialize(timer: &'static mut Timer, interrupt_handler: InterruptHandler) -> AlarmState {
    fn initialize(timer: &'static mut Timer, interrupt_handler: InterruptHandler) -> Alarm {
        // If the driver is initialized, bind the interrupt handler to the
        // timer. This ensures that alarms allocated after init are correctly
        // bound to the core that created the executor.
        timer.set_interrupt_handler(interrupt_handler);
        timer.enable_interrupt(true);
        //AlarmState::Initialized(timer)
        Alarm::Initialized(timer)
    }
}

// 79
impl Alarm {
    // 80
    pub const fn new(handler: extern "C" fn()) -> Self {
        Alarm::Created(handler)
    }
}

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
///
/// I've chosed this implementation above but w/o 'sync and mutex' stuff.
/// So 'RawQueue' of 'TimerQueue' in timer_queue.rs is rally a copy of 'Queue'
/// in queue_integrated.rs in embassy-time-queue-utils crate.
///
/// - If the `multiple-integrated` feature is enabled, we provide a separate
///   timer queue for each executor. We store a separate timer queue for each
///   executor, and we use the scheduled task's owner to determine which queue
///   to use. This mode allows us to use less disruptive locks around the timer
///   queue, but requires more timers - one per timer queue.
// 109
pub(super) struct EmbassyTimer {
    /// The timer queue, if we use a single one (single-integrated, or generic).
    #[cfg(single_queue)]
    pub(crate) inner: crate::timer_queue::TimerQueue,
    alarms: [Alarm; MAX_SUPPORTED_ALARM_COUNT],
    //available_timers: Locked<Option<&'static mut [Timer]>>,
    available_timers: Option<&'static mut [Timer]>,
}

/// Repeats the `Alarm::new` constructor for each alarm, creating an interrupt
/// handler for each of them.
// 120
macro_rules! alarms {
    ($($idx:literal),*) => {
        [$(
            Alarm::new({
                // Not #[handler] so we don't have to store the priority - which is constant.
                extern "C" fn handler() {
                    unsafe {
                        DRIVER.on_interrupt($idx);
                    }
                }
                handler
            })
        ),*]
    };
}

//const MAX_SUPPORTED_ALARM_COUNT: usize = 7;
// we have only one Timer yet
// 137
const MAX_SUPPORTED_ALARM_COUNT: usize = 1;

// 139
//embassy_time_driver::time_driver_impl!(static DRIVER: EmbassyTimer = EmbassyTimer {
embassy_time_driver::time_driver_impl!(static mut DRIVER: EmbassyTimer = EmbassyTimer {
    // Single queue, needs maximum priority.
    #[cfg(single_queue)]
    inner: crate::timer_queue::TimerQueue::new(Priority::max()),
    //alarms: alarms!(0, 1, 2, 3, 4, 5, 6),
    alarms: alarms!(0),
    //available_timers: Locked::new(None),
    available_timers: None,
});

// 147
impl EmbassyTimer {
    // 148
    pub(super) fn init(timers: &'static mut [Timer]) {
        // Reset timers
        timers.iter_mut().for_each(|timer| {
            timer.enable_interrupt(false);
            timer.stop();
        });

        // Store the available timers
        //DRIVER.available_timers.with(|available_timers| {
        //  *available_timers = Some(timers);
        unsafe {
            DRIVER.available_timers = Some(timers);
        }
        //});
    }

    // 178
    //fn on_interrupt(&self, id: usize) {
    fn on_interrupt(&mut self, id: usize) {
        // On interrupt, we clear the alarm that was triggered...
        #[cfg_attr(single_queue, allow(clippy::let_unit_value))]
        //let _ctx = self.alarms[id].inner.with(|alarm| {
        //if let AlarmState::Initialized(timer) = &mut alarm.state {
        if let Alarm::Initialized(timer) = &mut self.alarms[id] {
            timer.clear_interrupt();
            #[cfg(not(single_queue))]
            alarm.context.get()
        } else {
            //unsafe {
            // SAFETY: `on_interrupt` is registered right when the alarm is initialized.
            //core::hint::unreachable_unchecked()
            unreachable!()
            //}
        }
        //});

        // ... and process the timer queue if we have one. For multiple queues, the
        // timer queue is stored in the alarm's context.
        #[cfg(all(integrated_timers, not(single_queue)))]
        {
            let executor = unsafe { &*_ctx.cast::<crate::executor::InnerExecutor>() };
            executor.timer_queue.dispatch();
        }

        // If we have a single queue, it lives in this struct.
        #[cfg(single_queue)]
        self.inner.dispatch();
    }

    /// Returns `true` if the timer was armed, `false` if the timestamp is in
    /// the past.
    // 209
    fn arm(timer: &mut Timer, timestamp: u64) -> bool {
        let now = Instant::now().duration_since_epoch().as_micros();

        if timestamp > now {
            let mut timeout = Duration::from_micros(timestamp - now);
            loop {
                // The timer API doesn't let us query a maximum timeout, so let's try backing
                // off on failure.
                match timer.schedule(timeout) {
                    Ok(()) => break,
                    Err(Error::InvalidTimeout) => {
                        // It's okay to wake up earlier than scheduled.
                        timeout = timeout / 2;
                        assert_ne!(timeout, Duration::ZERO);
                    }
                    other => unwrap!(other),
                }
            }
            true
        } else {
            // If the timestamp is past, we return `false` to ask embassy to poll again
            // immediately.
            timer.stop();
            false
        }
    }

    /// Allocate an alarm, if possible.
    ///
    /// Returns `None` if there are no available alarms.
    ///
    /// When using multiple timer queues, the `priority` parameter indicates the
    /// priority of the interrupt handler. It is 1 for thread-mode
    /// executors, or equals to the priority of an interrupt executor.
    ///
    /// When using a single timer queue, the `priority` parameter is always the
    /// highest value possible.
    // 246
    pub(crate) unsafe fn allocate_alarm(&mut self, priority: Priority) -> Option<AlarmHandle> {
        unsafe {
            //for (i, alarm) in self.alarms.iter().enumerate() {
            for (i, alarm) in self.alarms.iter_mut().enumerate() {
                //let handle = alarm.inner.with(|alarm| {
                let handle = {
                    //let AlarmState::Created(interrupt_handler) = alarm.state else {
                    let Alarm::Created(interrupt_handler) = *alarm else {
                        return None;
                    };

                    let timer = /*self.available_timers.with(|available_timers|*/ {
                    //if let Some(timers) = available_timers.take() {
                    if let Some(timers) = self.available_timers.take() {
                        // If the driver is initialized, we can allocate a timer.
                        // If this fails, we can't do anything about it.
                        let Some((timer, rest)) = timers.split_first_mut() else {
                            not_enough_timers();
                        };
                        self.available_timers = Some(rest);
                        timer
                    } else {
                        panic!("schedule_wake called before esp_hal_embassy::init()");
                    }

                //});
                };

                    //alarm.state = AlarmState::initialize(
                    *alarm = Alarm::initialize(
                        timer,
                        InterruptHandler::new(interrupt_handler, priority),
                    );

                    Some(AlarmHandle::new(i))

                    //});
                };

                if handle.is_some() {
                    return handle;
                }
            }

            None
        }
    }

    /// Set an alarm to fire at a certain timestamp.
    ///
    /// Returns `false` if the timestamp is in the past.
    // 288
    fn set_alarm(&mut self, alarm: AlarmHandle, timestamp: u64) -> bool {
        let alarm = &mut self.alarms[alarm.id];

        // The hardware fires the alarm even if timestamp is lower than the current
        // time. In this case the interrupt handler will pend a wake-up when we exit the
        // critical section.
        //
        // This is correct behavior. See https://docs.rs/embassy-time-driver/0.1.0/embassy_time_driver/trait.Driver.html#tymethod.set_alarm
        // (... the driver should return true and arrange to call the alarm callback as
        // soon as possible, but not synchronously.)

        //alarm.inner.with(|alarm| {
        //if let AlarmState::Initialized(timer) = &mut alarm.state {
        if let Alarm::Initialized(timer) = alarm {
            Self::arm(*timer, timestamp)
        } else {
            //unsafe {
            // SAFETY: We only create `AlarmHandle` instances after the alarm is
            // initialized.
            //core::hint::unreachable_unchecked()
            unreachable!()
            //}
        }
        //})
    }
}

// 313
impl Driver for EmbassyTimer {
    // 314
    fn now(&self) -> u64 {
        Instant::now().duration_since_epoch().as_micros()
    }

    // 318
    fn schedule_wake(&mut self, at: u64, waker: &core::task::Waker) {
        #[cfg(single_queue)]
        self.inner.schedule_wake(at, waker);
    }
}

#[cold]
#[track_caller]
// 379
fn not_enough_timers() -> ! {
    // This is wrapped in a separate function because rustfmt does not like
    // extremely long strings. Also, if log is used, this avoids storing the string
    // twice.
    panic!(
        "There are not enough timers to allocate a new alarm. Call esp_hal_embassy::init() with the correct number of timers, or consider either using the `single-integrated` or the `generic` timer queue flavors."
    );
}

// 388
//pub(crate) fn set_up_alarm(priority: Priority, _ctx: *mut ()) -> AlarmHandle {
pub(crate) fn set_up_alarm(priority: Priority) -> AlarmHandle {
    let alarm = unwrap!(
        unsafe { DRIVER.allocate_alarm(priority) },
        //.unwrap_or_else(|| not_enough_timers())
        "esp_hal_embassy::init() should be called before allocating alarm"
    );
    #[cfg(not(single_queue))]
    DRIVER.set_callback_ctx(alarm, _ctx);
    alarm
}
