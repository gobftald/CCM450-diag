// 10
use core::ptr::addr_of_mut;

// 15
use crate::compat::{
    common::{
        ConcurrentQueue, create_queue, create_recursive_mutex, delete_queue, lock_mutex,
        unlock_mutex,
    },
    malloc::calloc,
};

// 43
static mut QUEUE_HANDLE: *mut ConcurrentQueue = core::ptr::null_mut();

/// **************************************************************************
/// Name: esp_spin_lock_create
///
/// Description:
///   Create spin lock in SMP mode
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   Spin lock data pointer
///
/// *************************************************************************
// 180
pub unsafe extern "C" fn spin_lock_create() -> *mut crate::binary::c_types::c_void {
    let ptr = crate::compat::common::sem_create(1, 1);

    trace!("spin_lock_create {:?}", ptr);
    ptr as *mut crate::binary::c_types::c_void
}

/// **************************************************************************
/// Name: esp_spin_lock_delete
///
/// Description:
///   Delete spin lock
///
/// Input Parameters:
///   lock - Spin lock data pointer
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 200
pub unsafe extern "C" fn spin_lock_delete(lock: *mut crate::binary::c_types::c_void) {
    trace!("spin_lock_delete {:?}", lock);

    crate::compat::common::sem_delete(lock);
}

/// **************************************************************************
/// Name: esp_recursive_mutex_create
///
/// Description:
///   Create recursive mutex
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   Recursive mutex data pointer
///
/// *************************************************************************
// 319
pub unsafe extern "C" fn recursive_mutex_create() -> *mut crate::binary::c_types::c_void {
    create_recursive_mutex()
}

/// **************************************************************************
/// Name: esp_mutex_delete
///
/// Description:
///   Delete mutex
///
/// Input Parameters:
///   mutex_data - mutex data pointer
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 336
pub unsafe extern "C" fn mutex_delete(mutex: *mut crate::binary::c_types::c_void) {
    crate::compat::common::mutex_delete(mutex);
}

/// **************************************************************************
/// Name: esp_mutex_lock
///
/// Description:
///   Lock mutex
///
/// Input Parameters:
///   mutex_data - mutex data pointer
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
// 353
pub unsafe extern "C" fn mutex_lock(mutex: *mut crate::binary::c_types::c_void) -> i32 {
    lock_mutex(mutex)
}

/// **************************************************************************
/// Name: esp_mutex_unlock
///
/// Description:
///   Unlock mutex
///
/// Input Parameters:
///   mutex_data - mutex data pointer
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
// 370
pub unsafe extern "C" fn mutex_unlock(mutex: *mut crate::binary::c_types::c_void) -> i32 {
    unlock_mutex(mutex)
}

/// **************************************************************************
/// Name: esp_task_get_current_task
///
/// Description:
///   Transform from millim seconds to system ticks
///
/// Input Parameters:
///   ms - Millim seconds
///
/// Returned Value:
///   System ticks
///
/// *************************************************************************
// 786
pub unsafe extern "C" fn task_get_current_task() -> *mut crate::binary::c_types::c_void {
    let res = crate::preempt::current_task() as *mut crate::binary::c_types::c_void;
    trace!("task get current task - return {:?}", res);

    res
}

/// **************************************************************************
/// Name: esp_task_get_max_priority
///
/// Description:
///   Get OS task maximum priority
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   Task maximum priority
///
/// *************************************************************************
// 806
pub unsafe extern "C" fn task_get_max_priority() -> i32 {
    trace!("task_get_max_priority");
    255
}

/// **************************************************************************
/// Name: esp_malloc
///
/// Description:
///   Allocate a block of memory
///
/// Input Parameters:
///   size - memory size
///
/// Returned Value:
///   Memory pointer
///
/// *************************************************************************
// 824
pub unsafe extern "C" fn malloc(size: usize) -> *mut crate::binary::c_types::c_void {
    unsafe { crate::compat::malloc::malloc(size).cast() }
}

/// **************************************************************************
/// Name: esp_free
///
/// Description:
///   Free a block of memory
///
/// Input Parameters:
///   ptr - memory block
///
/// Returned Value:
///   No
///
/// *************************************************************************
// 841
pub unsafe extern "C" fn free(p: *mut crate::binary::c_types::c_void) {
    unsafe {
        crate::compat::malloc::free(p.cast());
    }
}

/// **************************************************************************
/// Name: esp_malloc_internal
///
/// Description:
///   Drivers allocate a block of memory
///
/// Input Parameters:
///   size - memory size
///
/// Returned Value:
///   Memory pointer
///
/// *************************************************************************
// 1537
pub unsafe extern "C" fn malloc_internal(size: usize) -> *mut crate::binary::c_types::c_void {
    unsafe { crate::compat::malloc::malloc(size).cast() }
}

/// **************************************************************************
/// Name: esp_calloc_internal
///
/// Description:
///   Drivers allocate some continuous blocks of memory
///
/// Input Parameters:
///   n    - memory block number
///   size - memory block size
///
/// Returned Value:
///   New memory pointer
///
/// *************************************************************************
// 1576
pub unsafe extern "C" fn calloc_internal(
    n: usize,
    size: usize,
) -> *mut crate::binary::c_types::c_void {
    unsafe { calloc(n as u32, size) as *mut crate::binary::c_types::c_void }
}

/// **************************************************************************
/// Name: esp_wifi_create_queue
///
/// Description:
///   Create Wi-Fi static message queue
///
/// Input Parameters:
///   queue_len - queue message number
///   item_size - message size
///
/// Returned Value:
///   Wi-Fi static message queue data pointer
///
/// *************************************************************************
// 1688
pub unsafe extern "C" fn wifi_create_queue(
    queue_len: crate::binary::c_types::c_int,
    item_size: crate::binary::c_types::c_int,
) -> *mut crate::binary::c_types::c_void {
    unsafe {
        let queue = create_queue(queue_len, item_size);
        QUEUE_HANDLE = queue;

        addr_of_mut!(QUEUE_HANDLE).cast()
    }
}

/// **************************************************************************
/// Name: esp_wifi_delete_queue
///
/// Description:
///   Delete Wi-Fi static message queue
///
/// Input Parameters:
///   queue - Wi-Fi static message queue data pointer
///
/// Returned Value:
///   None
///
/// *************************************************************************
pub unsafe extern "C" fn wifi_delete_queue(queue: *mut crate::binary::c_types::c_void) {
    trace!("wifi_delete_queue {:?}", queue);
    unsafe {
        if core::ptr::eq(queue, addr_of_mut!(QUEUE_HANDLE).cast()) {
            delete_queue(QUEUE_HANDLE);
        } else {
            warn!("unknown queue when trying to delete WIFI queue");
        }
    }
}
