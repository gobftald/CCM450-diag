#![no_std]

/// Alarm handle, assigned by the driver.
#[derive(Clone, Copy)]
// 75
pub struct AlarmHandle {
    id: u8,
}

/// Time driver
// 96
pub trait Driver: Send + Sync + 'static {
    /// Return the current timestamp in ticks.
    ///
    /// Implementations MUST ensure that:
    /// - This is guaranteed to be monotonic, i.e. a call to now() will always return
    ///   a greater or equal value than earlier calls. Time can't "roll backwards".
    /// - It "never" overflows. It must not overflow in a sufficiently long time frame, say
    ///   in 10_000 years (Human civilization is likely to already have self-destructed
    ///   10_000 years from now.). This means if your hardware only has 16bit/32bit timers
    ///   you MUST extend them to 64-bit, for example by counting overflows in software,
    ///   or chaining multiple timers together.
    // 7
    fn now(&self) -> u64;

    /// Try allocating an alarm handle. Returns None if no alarms left.
    /// Initially the alarm has no callback set, and a null `ctx` pointer.
    // 114
    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle>;

    /// Set the callback function to be called when the alarm triggers.
    /// The callback may be called from any context (interrupt or thread mode).
    // 118
    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ());

    /// Set an alarm at the given timestamp.
    ///
    /// ## Behavior
    ///
    /// If `timestamp` is in the future, `set_alarm` schedules calling the callback function
    /// at that time, and returns `true`.
    ///
    /// If `timestamp` is in the past, `set_alarm` has two allowed behaviors. Implementations can pick whether to:
    ///
    /// - Schedule calling the callback function "immediately", as if the requested timestamp was "now+epsilon" and return `true`, or
    /// - Not schedule the callback, and return `false`.
    ///
    /// Callers must ensure to behave correctly with either behavior.
    ///
    /// When callback is called, it is guaranteed that `now()` will return a value greater than or equal to `timestamp`.
    ///
    /// ## Reentrancy
    ///
    /// Calling the callback from `set_alarm` synchronously is not allowed. If the implementation chooses the first option above,
    /// it must still call the callback from another context (i.e. an interrupt handler or background thread), it's not allowed
    /// to call it synchronously in the context `set_alarm` is running.
    ///
    /// The reason for the above is callers are explicitly permitted to do both of:
    /// - Lock a mutex in the alarm callback.
    /// - Call `set_alarm` while having that mutex locked.
    ///
    /// If `set_alarm` called the callback synchronously, it'd cause a deadlock or panic because it'd cause the
    /// mutex to be locked twice reentrantly in the same context.
    ///
    /// ## Overwriting alarms
    ///
    /// Only one alarm can be active at a time for each `AlarmHandle`. This overwrites any previously-set alarm if any.
    ///
    /// ## Unsetting the alarm
    ///
    /// There is no `unset_alarm` API. Instead, callers can call `set_alarm` with `timestamp` set to `u64::MAX`.
    ///
    /// This allows for more efficient implementations, since they don't need to distinguish between the "alarm set" and
    /// "alarm not set" cases, thanks to the fact "Alarm set for u64::MAX" is effectively equivalent for "alarm not set".
    ///
    /// This means implementations need to be careful to avoid timestamp overflows. The recommendation is to make `timestamp`
    /// be in the same units as hardware ticks to avoid any conversions, which makes avoiding overflow easier.
    // 162
    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool;
}

// 165
extern "Rust" {
    fn _embassy_time_now() -> u64;
    fn _embassy_time_allocate_alarm() -> Option<AlarmHandle>;
    fn _embassy_time_set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ());
    fn _embassy_time_set_alarm(alarm: AlarmHandle, timestamp: u64) -> bool;
}

/// See [`Driver::now`]
// 173
pub fn now() -> u64 {
    unsafe { _embassy_time_now() }
}

/// See [`Driver::allocate_alarm`]
// 177
pub unsafe fn allocate_alarm() -> Option<AlarmHandle> {
    _embassy_time_allocate_alarm()
}

/// See [`Driver::set_alarm_callback`]
// 185
pub fn set_alarm_callback(alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
    unsafe { _embassy_time_set_alarm_callback(alarm, callback, ctx) }
}

/// See [`Driver::set_alarm`]
// 190
pub fn set_alarm(alarm: AlarmHandle, timestamp: u64) -> bool {
    unsafe { _embassy_time_set_alarm(alarm, timestamp) }
}

/// Set the time Driver implementation.
///
#[macro_export]
// 198
macro_rules! time_driver_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[no_mangle]
        fn _embassy_time_now() -> u64 {
            <$t as $crate::Driver>::now(&$name)
        }

        #[no_mangle]
        unsafe fn _embassy_time_allocate_alarm() -> Option<$crate::AlarmHandle> {
            <$t as $crate::Driver>::allocate_alarm(&$name)
        }

        #[no_mangle]
        fn _embassy_time_set_alarm_callback(
            alarm: $crate::AlarmHandle,
            callback: fn(*mut ()),
            ctx: *mut (),
        ) {
            <$t as $crate::Driver>::set_alarm_callback(&$name, alarm, callback, ctx)
        }

        #[no_mangle]
        fn _embassy_time_set_alarm(alarm: $crate::AlarmHandle, timestamp: u64) -> bool {
            <$t as $crate::Driver>::set_alarm(&$name, alarm, timestamp)
        }
    };
}
