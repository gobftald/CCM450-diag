//! This module allows hooking `esp-wifi` into an external scheduler, instead
//! of using the integrated one as provided by the `preempt` module.

// 21
use core::ffi::c_void;

// 23
//pub trait Scheduler: Send + Sync + 'static {
pub trait Scheduler: 'static {
    /// This function is called by `esp-wifi` when starting up the WiFi stack.
    // 25
    fn enable(&self);

    // 31
    /// This function is called by threads and should switch to the next thread.
    fn yield_task(&self);

    /// This function is called by threads and should return an opaque handle
    /// for the calling thread. The same handle will be passed to
    /// `esp_wifi_preempt_schedule_task_deletion`.
    // 36
    fn current_task(&self) -> *mut c_void;

    /// This function is used to create threads.
    /// It should allocate the stack.
    // 40
    fn task_create(
        &self,
        task: extern "C" fn(*mut c_void),
        param: *mut c_void,
        task_stack_size: usize,
    ) -> *mut c_void;
}

// 59
unsafe extern "Rust" {
    fn esp_wifi_preempt_enable();
    fn esp_wifi_preempt_yield_task();
    fn esp_wifi_preempt_current_task() -> *mut c_void;
    fn esp_wifi_preempt_task_create(
        task: extern "C" fn(*mut c_void),
        param: *mut c_void,
        task_stack_size: usize,
    ) -> *mut c_void;
}

// 73
pub(crate) fn enable() {
    unsafe { esp_wifi_preempt_enable() }
}

// 85
pub(crate) fn current_task() -> *mut c_void {
    unsafe { esp_wifi_preempt_current_task() }
}

// 89
pub(crate) fn task_create(
    task: extern "C" fn(*mut c_void),
    param: *mut c_void,
    task_stack_size: usize,
) -> *mut c_void {
    unsafe { esp_wifi_preempt_task_create(task, param, task_stack_size) }
}

// 81
pub(crate) fn yield_task() {
    unsafe { esp_wifi_preempt_yield_task() }
}

/// Set the Scheduler implementation.
///
/// See the module documentation for an example.
#[macro_export]
// 109
macro_rules! scheduler_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        // 111
        static $name: $t = $val;

        #[unsafe(no_mangle)]
        // 114
        fn esp_wifi_preempt_enable() {
            <$t as $crate::preempt::Scheduler>::enable(&$name)
        }

        #[unsafe(no_mangle)]
        // 122
        fn esp_wifi_preempt_yield_task() {
            <$t as $crate::preempt::Scheduler>::yield_task(&$name)
        }

        #[unsafe(no_mangle)]
        // 126
        fn esp_wifi_preempt_current_task() -> *mut c_void {
            <$t as $crate::preempt::Scheduler>::current_task(&$name)
        }

        #[unsafe(no_mangle)]
        // 130
        fn esp_wifi_preempt_task_create(
            task: extern "C" fn(*mut c_void),
            param: *mut c_void,
            task_stack_size: usize,
        ) -> *mut c_void {
            <$t as $crate::preempt::Scheduler>::task_create(&$name, task, param, task_stack_size)
        }
    };
}
