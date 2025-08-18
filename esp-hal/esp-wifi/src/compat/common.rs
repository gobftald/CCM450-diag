#![allow(unused)]

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

    // 42
    pub(crate) fn enqueue(&mut self, item: *mut c_void) -> i32 {
        self.raw_queue.with(|q| unsafe { q.enqueue(item) })
    }

    // 46
    pub(crate) fn try_dequeue(&mut self, item: *mut c_void) -> bool {
        self.raw_queue.with(|q| unsafe { q.try_dequeue(item) })
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

    // 86
    fn get(&self, index: usize) -> &[u8] {
        let item_start = self.item_size * index;
        &self.storage[item_start..][..self.item_size]
    }

    // 91
    fn get_mut(&mut self, index: usize) -> &mut [u8] {
        let item_start = self.item_size * index;
        &mut self.storage[item_start..][..self.item_size]
    }

    // 96
    fn full(&self) -> bool {
        self.count() == self.capacity
    }

    // 100
    fn empty(&self) -> bool {
        self.count() == 0
    }

    // 104
    unsafe fn enqueue(&mut self, item: *mut c_void) -> i32 {
        if !self.full() {
            let item = unsafe { core::slice::from_raw_parts(item as *const u8, self.item_size) };

            let dst = self.get_mut(self.current_write);
            dst.copy_from_slice(item);

            self.current_write = (self.current_write + 1) % self.capacity;
            1
        } else {
            0
        }
    }

    // 118
    unsafe fn try_dequeue(&mut self, item: *mut c_void) -> bool {
        if !self.empty() {
            let item = unsafe { core::slice::from_raw_parts_mut(item as *mut u8, self.item_size) };

            let src = self.get(self.current_read);
            item.copy_from_slice(src);

            self.current_read = (self.current_read + 1) % self.capacity;

            true
        } else {
            false
        }
    }

    // 158
    fn count(&self) -> usize {
        if self.current_write >= self.current_read {
            self.current_write - self.current_read
        } else {
            self.capacity - self.current_read + self.current_write
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

// 189
pub(crate) fn sem_create(_max: u32, init: u32) -> *mut c_void {
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

// 207
pub(crate) fn sem_take(semphr: *mut c_void, tick: u32) -> i32 {
    // This shouldn't normally happen if we always report the correct state from
    // `is_in_isr`. This is a last resort if the driver calls this anyways.
    // (I haven't observed this to happen)
    let tick = if tick == OSI_FUNCS_TIME_BLOCKING && crate::is_interrupts_disabled() {
        warn!("blocking sem_take probably called from an ISR - return early");
        1
    } else {
        tick
    };

    //trace!(">>>> semphr_take {:?} block_time_tick {}", semphr, tick);

    let forever = tick == OSI_FUNCS_TIME_BLOCKING;
    let timeout = tick as u64;
    let start = crate::time::systimer_count();

    let sem = semphr as *mut u32;

    'outer: loop {
        let res = critical_section::with(|_| unsafe {
            memory_fence();
            let cnt = *sem;
            if cnt > 0 {
                *sem = cnt - 1;
                1
            } else {
                0
            }
        });

        if res == 1 {
            //trace!(">>>> return from semphr_take");
            return 1;
        }

        if !forever && crate::time::elapsed_time_since(start) > timeout {
            break 'outer;
        }

        yield_task();
    }

    trace!(">>>> return from semphr_take with timeout");
    0
}

// 254
pub(crate) fn sem_give(semphr: *mut c_void) -> i32 {
    //trace!("semphr_give {:?}", semphr);

    let sem = semphr as *mut u32;

    critical_section::with(|_| unsafe {
        let cnt = *sem;
        *sem = cnt + 1;
        1
    })
}

pub(crate) fn thread_sem_get() -> *mut c_void {
    //trace!("wifi_thread_semphr_get");
    crate::preempt::current_task_thread_semaphore()
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

    //trace!("recursive_mutex_create called {:?}", ptr);
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
    //trace!("mutex_lock ptr = {:?}", mutex);

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
    //trace!("mutex_unlock {:?}", mutex);

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

pub(crate) fn send_queued(
    queue: *mut ConcurrentQueue,
    item: *mut c_void,
    block_time_tick: u32,
) -> i32 {
    //trace!(
    //    "queue_send queue {:?} item {:x} block_time_tick {}",
    //    queue, item as usize, block_time_tick
    //);

    let queue: *mut ConcurrentQueue = queue.cast();
    unsafe { (*queue).enqueue(item) }
}

// 376
pub(crate) fn receive_queued(
    queue: *mut ConcurrentQueue,
    item: *mut c_void,
    block_time_tick: u32,
) -> i32 {
    //trace!(
    //    "queue_recv {:?} item {:?} block_time_tick {}",
    //    queue,
    //    item,
    //    block_time_tick
    //);

    let forever = block_time_tick == OSI_FUNCS_TIME_BLOCKING;
    let timeout = block_time_tick as u64;
    let start = crate::time::systimer_count();

    loop {
        if unsafe { (*queue).try_dequeue(item) } {
            //trace!("received");
            return 1;
        }

        if !forever && crate::time::elapsed_time_since(start) > timeout {
            trace!("queue_recv returns with timeout");
            return -1;
        }

        yield_task();
    }
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
