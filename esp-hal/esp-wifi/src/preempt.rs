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

pub(crate) fn task_create(
    task: extern "C" fn(*mut c_void),
    param: *mut c_void,
    task_stack_size: usize,
) -> *mut c_void {
    unsafe { esp_wifi_preempt_task_create(task, param, task_stack_size) }
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
