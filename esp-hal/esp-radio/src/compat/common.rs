#![allow(unused)]

// 10
use allocator_api2::boxed::Box;
use esp_wifi_sys_esp32c3::{c_types::c_char, include::malloc};

// 13
use super::malloc::free;
use crate::{
    binary::c_types::{c_int, c_uint, c_void},
    compat::malloc::InternalMemory,
    memory_fence::memory_fence,
    preempt::{current_task, yield_task},
};
use esp_sync::NonReentrantMutex;

// 22
pub(crate) const OSI_FUNCS_TIME_BLOCKING: u32 = u32::MAX;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 25
struct Mutex {
    locking_pid: usize,
    count: u32,
    recursive: bool,
}

// 167
pub unsafe fn str_from_c<'a>(s: *const c_char) -> &'a str {
    unsafe {
        let c_str = core::ffi::CStr::from_ptr(s.cast());
        core::str::from_utf8_unchecked(c_str.to_bytes())
    }
}

#[unsafe(no_mangle)]
// 175
unsafe extern "C" fn strnlen(chars: *const c_char, _maxlen: usize) -> usize {
    let mut len = 0;
    loop {
        unsafe {
            if chars.offset(len).read_volatile() == 0 {
                break;
            }
            len += 1;
        }
    }

    len as usize
}

// 37
pub(crate) fn thread_sem_get() -> *mut c_void {
    //trace!("wifi_thread_semphr_get");
    let ptr = crate::preempt::current_task_thread_semaphore()
        .as_ptr()
        .cast::<c_void>();
    trace!("thread_semp_get {}", ptr);
    ptr
}

/// Implementation of sleep() from newlib in esp-idf.
/// components/newlib/time.c
#[unsafe(no_mangle)]
// 415
pub(crate) unsafe extern "C" fn sleep(
    seconds: crate::binary::c_types::c_uint,
) -> crate::binary::c_types::c_uint {
    trace!("sleep");

    unsafe {
        usleep(seconds * 1_000);
    }
    0
}

/// Implementation of usleep() from newlib in esp-idf.
/// components/newlib/time.c
#[unsafe(no_mangle)]
// 429
unsafe extern "C" fn usleep(us: u32) -> crate::binary::c_types::c_int {
    trace!("usleep");
    unsafe extern "C" {
        fn esp_rom_delay_us(us: u32);
    }

    unsafe {
        esp_rom_delay_us(us);
    }
    0
}

/// Implementation of usleep() from newlib in esp-idf.
/// components/newlib/time.c
#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn __esp_radio_usleep(us: u32) -> c_int {
    crate::preempt::usleep(us);

    0
}