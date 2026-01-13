use esp_radio_rtos_driver::semaphore::SemaphoreKind;

use crate::{
    preempt::semaphore::{SemaphoreHandle, SemaphorePtr},
    sys::c_types::c_void,
};

pub(crate) fn sem_create(max: u32, initial: u32) -> *mut c_void {
    let ptr = SemaphoreHandle::new(SemaphoreKind::Counting { max, initial })
        .leak()
        .as_ptr()
        .cast();

    trace!("sem_create -> {:?}", ptr);

    ptr
}

pub(crate) fn sem_delete(semphr: *mut c_void) {
    trace!("sem_delete: {:?}", semphr);
    let ptr = unwrap!(SemaphorePtr::new(semphr.cast()), "semphr is null");

    let handle = unsafe { SemaphoreHandle::from_ptr(ptr) };
    core::mem::drop(handle);
}
