// 10
use core::{cell::RefCell, ptr::addr_of_mut};

// 12
use enumset::EnumSet;

// 14
use super::WifiEvent;
use crate::{
    compat::{
        common::{
            ConcurrentQueue, create_queue, create_recursive_mutex, delete_queue, lock_mutex,
            receive_queued, send_queued, thread_sem_get, unlock_mutex,
        },
        malloc::calloc,
    },
    hal::{
        clock::RadioClockController,
        peripherals::RADIO_CLK,
        sync::{Locked, RawMutex},
    },
    memory_fence::memory_fence,
    preempt::yield_task,
};

static mut WIFI_LOCK: RawMutex = RawMutex::new();

// 43
static mut QUEUE_HANDLE: *mut ConcurrentQueue = core::ptr::null_mut();

// useful for waiting for events - clear and wait for the event bit to be set
// again
// 47
pub(crate) static WIFI_EVENTS: Locked<RefCell<EnumSet<WifiEvent>>> =
    Locked::new(RefCell::new(enumset::enum_set!()));

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
/// Name: esp_wifi_int_disable
///
/// Description:
///   Enter critical section by disabling interrupts and taking the spin lock
///   if in SMP mode.
///
/// Input Parameters:
///   wifi_int_mux - Spin lock data pointer
///
/// Returned Value:
///   CPU PS value.
///
/// *************************************************************************
// 220
pub unsafe extern "C" fn wifi_int_disable(
    _wifi_int_mux: *mut crate::binary::c_types::c_void,
) -> u32 {
    trace!("wifi_int_disable");
    // TODO: can we use wifi_int_mux?
    let token = unsafe { WIFI_LOCK.acquire() };
    unsafe { core::mem::transmute::<esp_hal::sync::RestoreState, u32>(token) }
}

/// **************************************************************************
/// Name: esp_wifi_int_restore
///
/// Description:
///   Exit from critical section by enabling interrupts and releasing the spin
///   lock if in SMP mode.
///
/// Input Parameters:
///   wifi_int_mux - Spin lock data pointer
///   tmp          - CPU PS value.
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 244
pub unsafe extern "C" fn wifi_int_restore(
    _wifi_int_mux: *mut crate::binary::c_types::c_void,
    tmp: u32,
) {
    trace!("wifi_int_restore");
    let token = unsafe { core::mem::transmute::<u32, esp_hal::sync::RestoreState>(tmp) };
    unsafe { WIFI_LOCK.release(token) }
}

/// **************************************************************************
/// Name: esp_thread_semphr_get
///
/// Description:
///   Get thread self's semaphore
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   Semaphore data pointer
///
/// *************************************************************************
// 285
pub unsafe extern "C" fn wifi_thread_semphr_get() -> *mut crate::binary::c_types::c_void {
    thread_sem_get()
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
/// Name: esp_queue_send
///
/// Description:
///   Send message of low priority to queue within a certain period of time
///
/// Input Parameters:
///   queue - Message queue data pointer
///   item  - Message data pointer
///   ticks - Wait ticks
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
// 436
pub unsafe extern "C" fn queue_send(
    queue: *mut crate::binary::c_types::c_void,
    item: *mut crate::binary::c_types::c_void,
    block_time_tick: u32,
) -> i32 {
    send_queued(queue.cast(), item, block_time_tick)
}

/// **************************************************************************
/// Name: esp_queue_recv
///
/// Description:
///   Receive message from queue within a certain period of time
///
/// Input Parameters:
///   queue - Message queue data pointer
///   item  - Message data pointer
///   ticks - Wait ticks
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
// 533
pub unsafe extern "C" fn queue_recv(
    queue: *mut crate::binary::c_types::c_void,
    item: *mut crate::binary::c_types::c_void,
    block_time_tick: u32,
) -> i32 {
    receive_queued(queue.cast(), item, block_time_tick)
}

/// **************************************************************************
/// Name: esp_task_create_pinned_to_core
///
/// Description:
///   Create task and bind it to target CPU, the task will run when it
///   is created
///
/// Input Parameters:
///   entry       - Task entry
///   name        - Task name
///   stack_depth - Task stack size
///   param       - Task private data
///   prio        - Task priority
///   task_handle - Task handle pointer which is used to pause, resume
///                 and delete the task
///   core_id     - CPU which the task runs in
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
#[allow(unused_variables)]
// 646
pub unsafe extern "C" fn task_create_pinned_to_core(
    task_func: *mut crate::binary::c_types::c_void,
    name: *const crate::binary::c_types::c_char,
    stack_depth: u32,
    param: *mut crate::binary::c_types::c_void,
    prio: u32,
    task_handle: *mut crate::binary::c_types::c_void,
    core_id: u32,
) -> i32 {
    trace!(
        "task_create_pinned_to_core task_func {:?} name {} stack_depth {} param {:?} prio {}, task_handle {:?} core_id {}",
        task_func,
        unsafe { crate::compat::common::str_from_c(name as _) },
        stack_depth,
        param,
        prio,
        task_handle,
        core_id
    );

    unsafe {
        let task_func = core::mem::transmute::<
            *mut crate::binary::c_types::c_void,
            extern "C" fn(*mut esp_wifi_sys::c_types::c_void),
        >(task_func);

        let task = crate::preempt::task_create(task_func, param, stack_depth as usize);
        *(task_handle as *mut usize) = task as usize;

        1
    }
}

/// **************************************************************************
/// Name: esp_task_delay
///
/// Description:
///   Current task wait for some ticks
///
/// Input Parameters:
///   tick - Waiting ticks
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 747
pub unsafe extern "C" fn task_delay(tick: u32) {
    trace!("task_delay tick {}", tick);
    let start_time = crate::time::systimer_count();
    while crate::time::elapsed_time_since(start_time) < tick as u64 {
        yield_task();
    }
}

/// **************************************************************************
/// Name: esp_task_ms_to_tick
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
// 768
pub unsafe extern "C" fn task_ms_to_tick(ms: u32) -> i32 {
    trace!("task_ms_to_tick ms {}", ms);
    crate::time::millis_to_ticks(ms as u64) as i32
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
/// Name: esp_event_post
///
/// Description:
///   Active work queue and let the work to process the cached event
///
/// Input Parameters:
///   event_base      - Event set name
///   event_id        - Event ID
///   event_data      - Event private data
///   event_data_size - Event data size
///   ticks           - Waiting system ticks
///
/// Returned Value:
///   0 if success or -1 if fail
///
/// *************************************************************************
// 864
pub unsafe extern "C" fn event_post(
    event_base: *const crate::binary::c_types::c_char,
    event_id: i32,
    event_data: *mut crate::binary::c_types::c_void,
    event_data_size: usize,
    ticks_to_wait: u32,
) -> i32 {
    trace!(
        "event_post {:?} {} {:?} {} {:?}",
        event_base, event_id, event_data, event_data_size, ticks_to_wait
    );
    use num_traits::FromPrimitive;

    let event = unwrap!(WifiEvent::from_i32(event_id));
    trace!("EVENT: {:?}", event);

    WIFI_EVENTS.with(|events| events.borrow_mut().insert(event));
    let handled =
        unsafe { super::event::dispatch_event_handler(event, event_data, event_data_size) };

    super::state::update_state(event, handled);

    /*
    event.waker().wake();

    match event {
        WifiEvent::StaConnected | WifiEvent::StaDisconnected => {
            crate::wifi::embassy::STA_LINK_STATE_WAKER.wake();
        }

        WifiEvent::ApStart | WifiEvent::ApStop => {
            crate::wifi::embassy::AP_LINK_STATE_WAKER.wake();
        }

        _ => {}
    }
    */

    memory_fence();

    0
}

/// **************************************************************************
/// Name: wifi_apb80m_request
///
/// Description:
///   Take Wi-Fi lock in auto-sleep
///
/// *************************************************************************
// 971
pub unsafe extern "C" fn wifi_apb80m_request() {
    trace!("wifi_apb80m_request - no-op")
}

/// **************************************************************************
/// Name: esp32c3_phy_enable
///
/// Description:
///   Initialize PHY hardware
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 1019
pub unsafe extern "C" fn phy_enable() {
    // quite some code needed here
    trace!("phy_enable");

    unsafe {
        crate::common_adapter::chip_specific::phy_enable();
    }
}

/// **************************************************************************
/// Name: wifi_reset_mac
///
/// Description:
///   Reset Wi-Fi hardware MAC
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   None
///
/// *************************************************************************
pub unsafe extern "C" fn wifi_reset_mac() {
    trace!("wifi_reset_mac");
    // stealing RADIO_CLK is safe since it is passed (as mutable reference or by
    // value) into `init`
    let radio_clocks = unsafe { RADIO_CLK::steal() };
    RadioClockController::new(radio_clocks).reset_mac();
}

/// **************************************************************************
/// Name: wifi_clock_enable
///
/// Description:
///   Enable Wi-Fi clock
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 1080
pub unsafe extern "C" fn wifi_clock_enable() {
    trace!("wifi_clock_enable");
    // stealing RADIO_CLK is safe since it is passed (as mutable reference or by
    // value) into `init`
    let radio_clocks = unsafe { RADIO_CLK::steal() };
    RadioClockController::new(radio_clocks).enable_wifi(true);
}

/// **************************************************************************
/// Name: esp_log_write
///
/// Description:
///   Output log with by format string and its arguments
///
/// Input Parameters:
///   level  - log level, no mean here
///   tag    - log TAG, no mean here
///   format - format string
///
/// Returned Value:
///   None
///
/// *************************************************************************
#[cfg(feature = "sys-logs")]
// 1465
pub unsafe extern "C" fn log_write(
    level: u32,
    _tag: *const crate::binary::c_types::c_char,
    format: *const crate::binary::c_types::c_char,
    args: ...
) {
    unsafe {
        crate::binary::log::syslog(level, format as _, args);
    }
}

/// **************************************************************************
/// Name: esp_log_writev
///
/// Description:
///   Output log with by format string and its arguments
///
/// Input Parameters:
///   level  - log level, no mean here
///   tag    - log TAG, no mean here
///   format - format string
///   args   - arguments list
///
/// Returned Value:
///   None
///
/// *************************************************************************
#[cfg(feature = "sys-logs")]
// 1492
pub unsafe extern "C" fn log_writev(
    level: u32,
    _tag: *const crate::binary::c_types::c_char,
    format: *const crate::binary::c_types::c_char,
    args: crate::binary::include::va_list,
) {
    unsafe {
        crate::binary::log::syslog(
            level,
            format as _,
            core::mem::transmute::<crate::binary::include::va_list, core::ffi::VaListImpl<'_>>(
                args,
            ),
        );
    }
}

/// **************************************************************************
/// Name: esp_log_timestamp
///
/// Description:
///   Get system time by millim second
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   System time
///
/// *************************************************************************
// 1518
pub unsafe extern "C" fn log_timestamp() -> u32 {
    esp_hal::time::Instant::now()
        .duration_since_epoch()
        .as_millis() as u32
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
/// Name: esp_zalloc_internal
///
/// Description:
///   Drivers allocate a block of memory and clear it with 0
///
/// Input Parameters:
///   size - memory size
///
/// Returned Value:
///   New memory pointer
///
/// *************************************************************************
// 1583
pub unsafe extern "C" fn zalloc_internal(size: usize) -> *mut crate::binary::c_types::c_void {
    unsafe { calloc(size as u32, 1usize) as *mut crate::binary::c_types::c_void }
}

/// **************************************************************************
/// Name: esp_wifi_malloc
///
/// Description:
///   Applications allocate a block of memory
///
/// Input Parameters:
///   size - memory size
///
/// Returned Value:
///   Memory pointer
///
/// *************************************************************************
// 1613
pub unsafe extern "C" fn wifi_malloc(size: usize) -> *mut crate::binary::c_types::c_void {
    unsafe { malloc(size) }
}

/// **************************************************************************
/// Name: esp_wifi_calloc
///
/// Description:
///   Applications allocate some continuous blocks of memory
///
/// Input Parameters:
///   n    - memory block number
///   size - memory block size
///
/// Returned Value:
///   New memory pointer
///
/// *************************************************************************
// 1652
pub unsafe extern "C" fn wifi_calloc(n: usize, size: usize) -> *mut crate::binary::c_types::c_void {
    trace!("wifi_calloc {} {}", n, size);
    unsafe { calloc(n as u32, size) as *mut crate::binary::c_types::c_void }
}

/// **************************************************************************
/// Name: esp_wifi_zalloc
///
/// Description:
///   Applications allocate a block of memory and clear it with 0
///
/// Input Parameters:
///   size - memory size
///
/// Returned Value:
///   New memory pointer
///
/// *************************************************************************
// 1670
pub unsafe extern "C" fn wifi_zalloc(size: usize) -> *mut crate::binary::c_types::c_void {
    unsafe { wifi_calloc(size, 1) }
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
// 1713
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

/// **************************************************************************
/// Name: wifi_coex_enable
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1747
pub unsafe extern "C" fn coex_enable() -> crate::binary::c_types::c_int {
    trace!("coex_enable");

    #[cfg(coex)]
    return unsafe { crate::binary::include::coex_enable() };

    #[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: esp_coex_wifi_request
///
/// Description:
///   Don't support
///
/// *************************************************************************
#[cfg_attr(not(coex), allow(unused_variables))]
// 1798
pub unsafe extern "C" fn coex_wifi_request(
    event: u32,
    latency: u32,
    duration: u32,
) -> crate::binary::c_types::c_int {
    trace!("coex_wifi_request");

    #[cfg(coex)]
    return unsafe { crate::binary::include::coex_wifi_request(event, latency, duration) };

    #[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: esp_coex_wifi_release
///
/// Description:
///   Don't support
///
/// *************************************************************************
#[cfg_attr(not(coex), allow(unused_variables))]
// 1820
pub unsafe extern "C" fn coex_wifi_release(event: u32) -> crate::binary::c_types::c_int {
    trace!("coex_wifi_release");

    #[cfg(coex)]
    return unsafe { crate::binary::include::coex_wifi_release(event) };

    #[cfg(not(coex))]
    0
}

#[allow(unused_variables)]
// 2020
pub unsafe extern "C" fn coex_schm_register_cb_wrapper(
    arg1: esp_wifi_sys::c_types::c_int,
    cb: ::core::option::Option<
        unsafe extern "C" fn(arg1: esp_wifi_sys::c_types::c_int) -> esp_wifi_sys::c_types::c_int,
    >,
) -> esp_wifi_sys::c_types::c_int {
    trace!("coex_schm_register_cb_wrapper {} {:?}", arg1, cb);

    //#[cfg(not(coex))]
    return 0;

    /*
    #[cfg(coex)]
    unsafe {
        crate::binary::include::coex_schm_register_callback(
            arg1 as u32,
            unwrap!(cb) as *const esp_wifi_sys::c_types::c_void
                as *mut esp_wifi_sys::c_types::c_void,
        )
    }
    */
}

// 2073
pub unsafe extern "C" fn coex_register_start_cb(
    _cb: Option<unsafe extern "C" fn() -> esp_wifi_sys::c_types::c_int>,
) -> esp_wifi_sys::c_types::c_int {
    //#[cfg(coex)]
    //return unsafe { esp_wifi_sys::include::coex_register_start_cb(_cb) };

    //#[cfg(not(coex))]
    0
}
