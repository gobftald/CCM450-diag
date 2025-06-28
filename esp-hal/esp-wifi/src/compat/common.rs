// 10
use allocator_api2::boxed::Box;

// 11
use esp_wifi_sys::{c_types::c_char, include::malloc};

// 13
use super::malloc::free;
use crate::{
    binary::c_types::{c_int, c_void},
    hal::sync::Locked,
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

// 31
pub(crate) struct ConcurrentQueue {
    raw_queue: Locked<RawQueue>,
}

// 35
impl ConcurrentQueue {
    // 36
    pub(crate) fn new(count: usize, item_size: usize) -> Self {
        Self {
            raw_queue: Locked::new(RawQueue::new(count, item_size)),
        }
    }
}

/// A naive and pretty much unsafe queue to back the queues used in drivers and
/// supplicant code.
///
/// The [ConcurrentQueue] wrapper should be used.
// 63
pub struct RawQueue {
    item_size: usize,
    capacity: usize,
    current_read: usize,
    current_write: usize,
    //storage: Box<[u8], InternalMemory>,
    storage: Box<[u8]>,
}

// 71
impl RawQueue {
    /// This allocates underlying storage. See [release_storage]
    // 73
    pub fn new(capacity: usize, item_size: usize) -> Self {
        let storage =
            //unsafe { Box::new_zeroed_slice_in(capacity * item_size, InternalMemory).assume_init() };
            unsafe { Box::new_zeroed_slice(capacity * item_size).assume_init() };

        Self {
            item_size,
            capacity,
            current_read: 0,
            current_write: 0,
            storage,
        }
    }
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

// 189
pub(crate) fn sem_create(max: u32, init: u32) -> *mut c_void {
    unsafe {
        let ptr = malloc(4) as *mut u32;
        ptr.write_volatile(init);

        trace!("sem created res = {:?}", ptr);
        ptr.cast()
    }
}

// 199
pub(crate) fn sem_delete(semphr: *mut c_void) {
    trace!(">>> sem delete");

    unsafe {
        free(semphr.cast());
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

// 287
pub(crate) fn mutex_delete(mutex: *mut c_void) {
    let ptr = mutex as *mut Mutex;
    unsafe {
        free(mutex.cast());
    }
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

// 339
pub(crate) fn create_queue(queue_len: c_int, item_size: c_int) -> *mut ConcurrentQueue {
    trace!("wifi_create_queue len={} size={}", queue_len, item_size,);

    let queue = ConcurrentQueue::new(queue_len as usize, item_size as usize);
    let ptr = unsafe { malloc(size_of_val(&queue) as u32) as *mut ConcurrentQueue };
    unsafe {
        ptr.write(queue);
    }

    trace!("created queue @{:?}", ptr);

    ptr
}

// 353
pub(crate) fn delete_queue(queue: *mut ConcurrentQueue) {
    trace!("delete_queue {:?}", queue);

    unsafe {
        core::ptr::drop_in_place(queue);
        crate::compat::malloc::free(queue.cast());
    }
}
