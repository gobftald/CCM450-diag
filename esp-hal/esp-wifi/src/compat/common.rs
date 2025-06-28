// 11
use esp_wifi_sys::{c_types::c_char, include::malloc};

// 14
use crate::{
    binary::c_types::c_void,
    memory_fence::memory_fence,
    preempt::{current_task, yield_task},
};

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

// 175
#[unsafe(no_mangle)]
unsafe extern "C" fn strnlen(chars: *const c_char, maxlen: usize) -> usize {
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

pub(crate) fn sem_create(max: u32, init: u32) -> *mut c_void {
    unsafe {
        let ptr = malloc(4) as *mut u32;
        ptr.write_volatile(init);

        trace!("sem created res = {:?}", ptr);
        ptr.cast()
    }
}

// 270
pub(crate) fn create_recursive_mutex() -> *mut c_void {
    let mutex = Mutex {
        locking_pid: 0xffff_ffff,
        count: 0,
        recursive: true,
    };

    let ptr = unsafe { malloc(size_of_val(&mutex) as u32) as *mut Mutex };
    unsafe {
        ptr.write(mutex);
    }
    memory_fence();

    trace!("recursive_mutex_create called {:?}", ptr);
    ptr as *mut c_void
}

/// Lock a mutex. Block until successful.
// 295
pub(crate) fn lock_mutex(mutex: *mut c_void) -> i32 {
    trace!("mutex_lock ptr = {:?}", mutex);

    let ptr = mutex as *mut Mutex;
    let current_task = current_task() as usize;

    loop {
        let mutex_locked = critical_section::with(|_| unsafe {
            if (*ptr).count == 0 {
                (*ptr).locking_pid = current_task;
                (*ptr).count += 1;
                true
            } else if (*ptr).locking_pid == current_task {
                (*ptr).count += 1;
                true
            } else {
                false
            }
        });
        memory_fence();

        if mutex_locked {
            return 1;
        }

        yield_task();
    }
}

// 324
pub(crate) fn unlock_mutex(mutex: *mut c_void) -> i32 {
    trace!("mutex_unlock {:?}", mutex);

    let ptr = mutex as *mut Mutex;
    critical_section::with(|_| unsafe {
        memory_fence();
        if (*ptr).count > 0 {
            (*ptr).count -= 1;
            1
        } else {
            0
        }
    })
}
