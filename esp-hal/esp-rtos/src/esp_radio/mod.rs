//! esp-radio support

// 3
use core::{ffi::c_void, ptr::NonNull};

// 10
use esp_radio_rtos_driver::semaphore::SemaphorePtr;

use crate::scheduler::Scheduler;

impl esp_radio_rtos_driver::Scheduler for Scheduler {
    fn initialized(&self) -> bool {
        false
    }

    fn yield_task(&self) {}

    fn yield_task_from_isr(&self) {}

    fn max_task_priority(&self) -> u32 {
        0
    }
    fn task_create(
        &self,
        name: &str,
        task: extern "C" fn(*mut c_void),
        param: *mut c_void,
        priority: u32,
        pin_to_core: Option<u32>,
        task_stack_size: usize,
    ) -> *mut c_void {
        core::ptr::null_mut() as *mut c_void
    }

    fn current_task(&self) -> *mut c_void {
        core::ptr::null_mut() as *mut c_void
    }

    fn schedule_task_deletion(&self, task_handle: *mut c_void) {}

    fn current_task_thread_semaphore(&self) -> SemaphorePtr {
        NonNull::<()>::dangling().cast()
    }

    fn usleep(&self, us: u32) {}

    fn now(&self) -> u64 {
        0
    }
}
