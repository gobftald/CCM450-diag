// 3
#[cfg_attr(esp32c3, path = "esp32c3.rs")]
pub(crate) mod os_adapter_chip_specific;

// 10
//use core::{cell::RefCell, ptr::addr_of_mut};
use allocator_api2::boxed::Box;

// 11
use enumset::EnumSet;
use esp_phy::PhyController;
use esp_sync::RawMutex;

// 15
use super::WifiEvent;
use crate::{
    binary::c_types::*,
    compat::{
        common::{str_from_c, thread_sem_get},
        malloc::{calloc_internal, InternalMemory},
    },
    hal::{
        clock::ModemClockController,
        //peripherals::RADIO_CLK,
        peripherals::WIFI,
        sync::NonReentrantMutex,
    },
    memory_fence::memory_fence,
    time::{blob_ticks_to_micros, millis_to_blob_ticks},
};

// 27
static WIFI_LOCK: RawMutex = RawMutex::new();

/*
// 43
static mut QUEUE_HANDLE: *mut ConcurrentQueue = core::ptr::null_mut();
*/

// useful for waiting for events - clear and wait for the event bit to be set
// again
// 31
//pub(crate) static WIFI_EVENTS: NonReentrantMutex<RefCell<EnumSet<WifiEvent>>> =
//    NonReentrantMutex::new(RefCell::new(enumset::enum_set!()));
pub(crate) static WIFI_EVENTS: NonReentrantMutex<EnumSet<WifiEvent>> =
    NonReentrantMutex::new(enumset::enum_set!());

/// **************************************************************************
/// Name: wifi_env_is_chip
///
/// Description:
///   Config chip environment
///
/// Returned Value:
///   True if on chip or false if on FPGA.
///
/// *************************************************************************
// 44
pub unsafe extern "C" fn env_is_chip() -> bool {
    true
}

/// **************************************************************************
/// Name: wifi_set_intr
///
/// Description:
///   Do nothing
///
/// Input Parameters:
///     cpu_no      - The CPU which the interrupt number belongs.
///     intr_source - The interrupt hardware source number.
///     intr_num    - The interrupt number CPU.
///     intr_prio   - The interrupt priority.
///
/// Returned Value:
///     None
///
/// *************************************************************************
// 64
pub unsafe extern "C" fn set_intr(cpu_no: i32, intr_source: u32, intr_num: u32, intr_prio: i32) {
    trace!(
        "set_intr {} {} {} {}",
        cpu_no,
        intr_source,
        intr_num,
        intr_prio
    );
    unsafe {
        crate::wifi::os_adapter::os_adapter_chip_specific::set_intr(
            cpu_no,
            intr_source,
            intr_num,
            intr_prio,
        );
    }
}

// 91
pub static mut ISR_INTERRUPT_1: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

/// **************************************************************************
/// Name: esp32c3_ints_on
///
/// Description:
///   Enable Wi-Fi interrupt
///
/// Input Parameters:
///   mask - No mean
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 107
pub unsafe extern "C" fn ints_on(mask: u32) {
    trace!("chip_ints_on {:x}", mask);

    crate::wifi::os_adapter::os_adapter_chip_specific::chip_ints_on(mask);
}

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
// 162
pub unsafe extern "C" fn spin_lock_create() -> *mut crate::binary::c_types::c_void {
    let ptr = crate::compat::semaphore::sem_create(1, 1);

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
// 182
pub unsafe extern "C" fn spin_lock_delete(lock: *mut crate::binary::c_types::c_void) {
    trace!("spin_lock_delete {:?}", lock);

    crate::compat::semaphore::sem_delete(lock);
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
// 202
pub unsafe extern "C" fn wifi_int_disable(
    _wifi_int_mux: *mut crate::binary::c_types::c_void,
) -> u32 {
    trace!("wifi_int_disable");
    // TODO: can we use wifi_int_mux?
    let token = unsafe { WIFI_LOCK.acquire() };
    //unsafe { core::mem::transmute::<esp_hal::sync::RestoreState, u32>(token) }
    unsafe { core::mem::transmute::<esp_sync::RestoreState, u32>(token) }
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
// 224
pub unsafe extern "C" fn wifi_int_restore(
    _wifi_int_mux: *mut crate::binary::c_types::c_void,
    tmp: u32,
) {
    trace!("wifi_int_restore");
    //let token = unsafe { core::mem::transmute::<u32, esp_hal::sync::RestoreState>(tmp) };
    let token = unsafe { core::mem::transmute::<u32, esp_sync::RestoreState>(tmp) };
    unsafe { WIFI_LOCK.release(token) }
}

/// **************************************************************************
/// Name: esp_task_yield_from_isr
///
/// Description:
///   Do nothing in NuttX
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   None
///
/// *************************************************************************
// 243
pub unsafe extern "C" fn task_yield_from_isr() {
    // original: /* Do nothing */
    trace!("task_yield_from_isr");
    //yield_task();
    crate::preempt::yield_task_from_isr();
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
// 261
pub unsafe extern "C" fn wifi_thread_semphr_get() -> *mut crate::binary::c_types::c_void {
    trace!("wifi_thread_semphr_get");
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
// 296
pub unsafe extern "C" fn recursive_mutex_create() -> *mut crate::binary::c_types::c_void {
    trace!("recursive_mutex_create");
    //create_recursive_mutex()
    crate::compat::mutex::mutex_create(true)
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
// 314
pub unsafe extern "C" fn mutex_delete(mutex: *mut crate::binary::c_types::c_void) {
    trace!("mutex_delete {}", mutex);
    //crate::compat::common::mutex_delete(mutex);
    crate::compat::mutex::mutex_delete(mutex);
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
// 331
pub unsafe extern "C" fn mutex_lock(mutex: *mut crate::binary::c_types::c_void) -> i32 {
    trace!("mutex_lock {:?}", mutex);
    //lock_mutex(mutex)
    crate::compat::mutex::mutex_lock(mutex)
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
// 348
pub unsafe extern "C" fn mutex_unlock(mutex: *mut crate::binary::c_types::c_void) -> i32 {
    trace!("mutex_unlock {:?}", mutex);
    //unlock_mutex(mutex)
    crate::compat::mutex::mutex_unlock(mutex)
}

/*
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
/// Name: esp_queue_send_from_isr
///
/// Description:
///   Send message of low priority to queue in ISR within
///   a certain period of time
///
/// Input Parameters:
///   queue - Message queue data pointer
///   item  - Message data pointer
///   hptw  - No mean
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
// 460
pub unsafe extern "C" fn queue_send_from_isr(
    queue: *mut crate::binary::c_types::c_void,
    item: *mut crate::binary::c_types::c_void,
    _hptw: *mut crate::binary::c_types::c_void,
) -> i32 {
    //trace!("queue_send_from_isr");
    unsafe {
        *(_hptw as *mut u32) = 1;
        queue_send(queue, item, 1000)
    }
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
*/

// 413
fn common_task_create(
    task_func: *mut c_void,
    name: *const c_char,
    stack_depth: u32,
    param: *mut c_void,
    prio: u32,
    task_handle: *mut c_void,
    core_id: Option<u32>,
) -> i32 {
    let task_name = unsafe { str_from_c(name as _) };
    trace!(
        "task_create task_func {:?} name {} stack_depth {} param {:?} prio {}, task_handle {:?} core_id {:?}",
        task_func, task_name, stack_depth, param, prio, task_handle, core_id
    );

    unsafe {
        let task_func = core::mem::transmute::<
            *mut c_void,
            extern "C" fn(*mut esp_wifi_sys_esp32c3::c_types::c_void),
        >(task_func);

        let task = crate::preempt::task_create(
            task_name,
            task_func,
            param,
            prio,
            core_id,
            stack_depth as usize,
        );
        *(task_handle as *mut usize) = task as usize;

        1
    }
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
// 469
pub unsafe extern "C" fn task_create_pinned_to_core(
    task_func: *mut c_void,
    name: *const c_char,
    stack_depth: u32,
    param: *mut c_void,
    prio: u32,
    task_handle: *mut c_void,
    core_id: u32,
) -> i32 {
    common_task_create(
        task_func,
        name,
        stack_depth,
        param,
        prio,
        task_handle,
        if core_id < 2 { Some(core_id) } else { None },
    )
}

/*  not needed yet
/// **************************************************************************
/// Name: esp_task_create
///
/// Description:
///   Create task and the task will run when it is created
///
/// Input Parameters:
///   entry       - Task entry
///   name        - Task name
///   stack_depth - Task stack size
///   param       - Task private data
///   prio        - Task priority
///   task_handle - Task handle pointer which is used to pause, resume
///                 and delete the task
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
// 508
pub unsafe extern "C" fn task_create(
    task_func: *mut c_void,
    name: *const c_char,
    stack_depth: u32,
    param: *mut c_void,
    prio: u32,
    task_handle: *mut c_void,
) -> i32 {
    common_task_create(task_func, name, stack_depth, param, prio, task_handle, None)
}
*/

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
// 554
pub unsafe extern "C" fn task_delay(tick: u32) {
    trace!("task_delay tick {}", tick);
    /*
    let start_time = crate::time::systimer_count();
    while crate::time::elapsed_time_since(start_time) < tick as u64 {
        yield_task();
    }
    */
    crate::preempt::usleep(blob_ticks_to_micros(tick))
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
// 572
pub unsafe extern "C" fn task_ms_to_tick(ms: u32) -> i32 {
    trace!("task_ms_to_tick ms {}", ms);
    //crate::time::millis_to_ticks(ms as u64) as i32
    millis_to_blob_ticks(ms) as i32
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
// 587
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
// 607
pub unsafe extern "C" fn task_get_max_priority() -> i32 {
    trace!("task_get_max_priority");
    //255
    crate::preempt::max_task_priority() as i32
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
// 625
pub unsafe extern "C" fn malloc(size: usize) -> *mut crate::binary::c_types::c_void {
    trace!("malloc {}", size);
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
// 642
pub unsafe extern "C" fn free(p: *mut crate::binary::c_types::c_void) {
    trace!("free {}", p);
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
#[allow(unused_variables)]
// 665
pub unsafe extern "C" fn event_post(
    event_base: *const crate::binary::c_types::c_char,
    event_id: i32,
    event_data: *mut crate::binary::c_types::c_void,
    event_data_size: usize,
    ticks_to_wait: u32,
) -> i32 {
    trace!(
        "event_post {:?} {} {:?} {} {:?}",
        event_base,
        event_id,
        event_data,
        event_data_size,
        ticks_to_wait
    );
    use num_traits::FromPrimitive;

    let event = unwrap!(WifiEvent::from_i32(event_id));
    trace!("EVENT: {:?}", event);

    //WIFI_EVENTS.with(|events| events.borrow_mut().insert(event));
    WIFI_EVENTS.with(|events| events.insert(event));

    let handled =
        unsafe { super::event::dispatch_event_handler(event, event_data, event_data_size) };

    super::state::update_state(event, handled);

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
// 769
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
// 814
pub unsafe extern "C" fn phy_enable() {
    // quite some code needed here
    trace!("phy_enable");

    //crate::common_adapter::chip_specific::phy_enable();
    core::mem::forget(unsafe { WIFI::steal() }.enable_phy());
}

/// **************************************************************************
/// Name: wifi_phy_update_country_info
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 827
#[allow(clippy::unnecessary_cast)]
#[allow(unused_variables)]
pub unsafe extern "C" fn phy_update_country_info(
    country: *const crate::binary::c_types::c_char,
) -> crate::binary::c_types::c_int {
    // not implemented in original code
    #[cfg(feature = "defmt")]
    unsafe {
        trace!("phy_update_country_info {}", str_from_c(country.cast()));
    }
    #[cfg(not(feature = "defmt"))]
    trace!("phy_update_country_info {}", unsafe { str_from_c(country.cast())});
    -1
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
// 849
pub unsafe extern "C" fn wifi_reset_mac() {
    trace!("wifi_reset_mac");
    /*
    // stealing RADIO_CLK is safe since it is passed (as mutable reference or by
    // value) into `init`
    let radio_clocks = unsafe { RADIO_CLK::steal() };
    RadioClockController::new(radio_clocks).reset_wifi_mac();
    */
    // stealing WIFI is safe, since it is passed into the initialization function of the BLE
    // controller.
    unsafe { WIFI::steal() }.reset_wifi_mac();
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
// 869
pub unsafe extern "C" fn wifi_clock_enable() {
    trace!("wifi_clock_enable");
    /*
    // stealing RADIO_CLK is safe since it is passed (as mutable reference or by
    // value) into `init`
    let radio_clocks = unsafe { RADIO_CLK::steal() };
    RadioClockController::new(radio_clocks).enable_wifi(true);
    */
    // stealing WIFI is safe, since it is passed into the initialization function of the BLE
    // controller.
    unsafe { WIFI::steal() }.enable_modem_clock(true);
}

/*
/// **************************************************************************
/// Name: esp_timer_get_time
///
/// Description:
///   Get time in microseconds since boot.
///
/// Returned Value:
///   System time in micros
///
/// *************************************************************************
#[unsafe(no_mangle)]
// 1142
pub unsafe extern "C" fn esp_timer_get_time() -> i64 {
    //trace!("esp_timer_get_time");
    crate::time::ticks_to_micros(crate::time::systimer_count()) as i64
}
*/

/// **************************************************************************
/// Name: esp_get_random
///
/// Description:
///   Fill random data int given buffer of given length
///
/// Input Parameters:
///   buf - buffer pointer
///   len - buffer length
///
/// Returned Value:
///   0 if success or -1 if fail
///
/// *************************************************************************
// 1178
pub unsafe extern "C" fn get_random(buf: *mut u8, len: usize) -> crate::binary::c_types::c_int {
    trace!("get_random");
    unsafe {
        //crate::common_adapter::esp_fill_random(buf, len as u32);
        crate::common_adapter::__esp_radio_esp_fill_random(buf, len as u32);
    }
    0
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
// 1218
#[cfg(feature = "sys-logs")]
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
// 1246
#[cfg(feature = "sys-logs")]
#[allow(improper_ctypes_definitions)]
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
// 1278
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
// 1297
pub unsafe extern "C" fn malloc_internal(size: usize) -> *mut crate::binary::c_types::c_void {
    //unsafe { crate::compat::malloc::malloc(size).cast() }
    unsafe { crate::compat::malloc::malloc_internal(size).cast() }
}

/* wifi_realloc does not used
/// **************************************************************************
/// Name: esp_realloc_internal
///
/// Description:
///   Drivers allocate a block of memory by old memory block
///
/// Input Parameters:
///   ptr  - old memory pointer
///   size - memory size
///
/// Returned Value:
///   New memory pointer
///
/// *************************************************************************
// 1315
pub unsafe extern "C" fn realloc_internal(ptr: *mut c_void, size: usize) -> *mut c_void {
    unsafe { crate::compat::malloc::realloc_internal(ptr.cast(), size).cast() }
}
*/

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
// 1333
pub unsafe extern "C" fn calloc_internal_wrapper(n: usize, size: usize) -> *mut c_void {
    trace!("calloc_internal_wrapper {} {}", n, size);
    unsafe { calloc_internal(n as u32, size) as *mut c_void }
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
// 1350
pub unsafe extern "C" fn zalloc_internal(size: usize) -> *mut crate::binary::c_types::c_void {
    trace!("zalloc_internal {}", size);
    //unsafe { calloc(size as u32, 1usize) as *mut crate::binary::c_types::c_void }
    unsafe { calloc_internal(size as u32, 1usize) as *mut c_void }
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
// 1367
pub unsafe extern "C" fn wifi_malloc(size: usize) -> *mut crate::binary::c_types::c_void {
    //unsafe { malloc(size) }
    unsafe { malloc_internal(size) }
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
// 1403
pub unsafe extern "C" fn wifi_calloc(n: usize, size: usize) -> *mut crate::binary::c_types::c_void {
    trace!("wifi_calloc {} {}", n, size);
    //unsafe { calloc(n as u32, size) as *mut crate::binary::c_types::c_void }
    unsafe { calloc_internal(n as u32, size) as *mut c_void }
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
// 1421
pub unsafe extern "C" fn wifi_zalloc(size: usize) -> *mut crate::binary::c_types::c_void {
    trace!("wifi_zalloc {}", size);
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
// 1439
pub unsafe extern "C" fn wifi_create_queue(
    queue_len: crate::binary::c_types::c_int,
    item_size: crate::binary::c_types::c_int,
) -> *mut crate::binary::c_types::c_void {
    trace!("wifi_create_queue len={} size={}", queue_len, item_size,);

    /*
    unsafe {
        let queue = create_queue(queue_len, item_size);
        QUEUE_HANDLE = queue;

        addr_of_mut!(QUEUE_HANDLE).cast()
    }
    */
    let queue = crate::compat::queue::queue_create(queue_len, item_size);

    let queue_ptr: *mut *mut c_void = Box::leak(Box::new_in(queue, InternalMemory));

    queue_ptr.cast()
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
// 1460
pub unsafe extern "C" fn wifi_delete_queue(queue: *mut crate::binary::c_types::c_void) {
    trace!("wifi_delete_queue {:?}", queue);
    /*
    unsafe {
        if core::ptr::eq(queue, addr_of_mut!(QUEUE_HANDLE).cast()) {
            delete_queue(QUEUE_HANDLE);
        } else {
            warn!("unknown queue when trying to delete WIFI queue");
        }
    }
    */
    let queue_ptr: *mut *mut c_void = queue.cast();

    let boxed = unsafe { Box::from_raw_in(queue_ptr, InternalMemory) };

    crate::compat::queue::queue_delete(*boxed)
}

/// **************************************************************************
/// Name: wifi_coex_enable
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1491
pub unsafe extern "C" fn coex_enable() -> crate::binary::c_types::c_int {
    trace!("coex_enable");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_enable() };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: esp_coex_status_get
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1524
pub unsafe extern "C" fn coex_status_get() -> u32 {
    trace!("coex_status_get");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_status_get() };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: esp_coex_wifi_request
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1542
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_wifi_request(
    event: u32,
    latency: u32,
    duration: u32,
) -> crate::binary::c_types::c_int {
    trace!("coex_wifi_request");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_wifi_request(event, latency, duration) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: esp_coex_wifi_release
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1559
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_wifi_release(event: u32) -> crate::binary::c_types::c_int {
    trace!("coex_wifi_release");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_wifi_release(event) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: wifi_coex_wifi_set_channel
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1577
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_wifi_channel_set(
    primary: u8,
    secondary: u8,
) -> crate::binary::c_types::c_int {
    trace!("coex_wifi_channel_set");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_wifi_channel_set(primary, secondary) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: wifi_coex_get_event_duration
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 11595
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_event_duration_get(
    event: u32,
    duration: *mut u32,
) -> crate::binary::c_types::c_int {
    trace!("coex_event_duration_get");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_event_duration_get(event, duration) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: wifi_coex_get_pti
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1613
#[cfg(any(esp32c3, esp32c2, esp32c6, esp32s3))]
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_pti_get(event: u32, pti: *mut u8) -> crate::binary::c_types::c_int {
    trace!("coex_pti_get");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_pti_get(event, pti) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: wifi_coex_clear_schm_status_bit
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1638
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_schm_status_bit_clear(type_: u32, status: u32) {
    trace!("coex_schm_status_bit_clear");

    //#[cfg(coex)]
    //unsafe {
    //    crate::binary::include::coex_schm_status_bit_clear(type_, status)
    //};
}

/// **************************************************************************
/// Name: wifi_coex_set_schm_status_bit
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1655
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_schm_status_bit_set(type_: u32, status: u32) {
    trace!("coex_schm_status_bit_set");

    //#[cfg(coex)]
    //unsafe {
    //    crate::binary::include::coex_schm_status_bit_set(type_, status)
    //};
}

/// **************************************************************************
/// Name: wifi_coex_set_schm_interval
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1672
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_schm_interval_set(interval: u32) -> crate::binary::c_types::c_int {
    trace!("coex_schm_interval_set");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_schm_interval_set(interval) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: wifi_coex_get_schm_curr_period
///
/// Description:
///   Don't support
///
/// *************************************************************************
// 1708
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_schm_curr_period_get() -> u8 {
    trace!("coex_schm_curr_period_get");

    //#[cfg(coex)]
    //return unsafe { crate::binary::include::coex_schm_curr_period_get() };

    //s#[cfg(not(coex))]
    0
}

// 1749
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_schm_register_cb_wrapper(
    arg1: esp_wifi_sys_esp32c3::c_types::c_int,
    cb: ::core::option::Option<
        unsafe extern "C" fn(
            arg1: esp_wifi_sys_esp32c3::c_types::c_int,
        ) -> esp_wifi_sys_esp32c3::c_types::c_int,
    >,
) -> esp_wifi_sys_esp32c3::c_types::c_int {
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

// 1771
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_schm_flexible_period_set(period: u8) -> i32 {
    trace!("coex_schm_flexible_period_set {}", period);

    /*
    #[cfg(coex)]
    unsafe {
        unsafe extern "C" {
            fn coex_schm_flexible_period_set(period: u8) -> i32;
        }

        coex_schm_flexible_period_set(period)
    }

    #[cfg(not(coex))]
    */
    0
}

// 1787
pub unsafe extern "C" fn coex_schm_flexible_period_get() -> u8 {
    trace!("coex_schm_flexible_period_get");

    /*
    #[cfg(coex)]
    unsafe {
        unsafe extern "C" {
            fn coex_schm_flexible_period_get() -> u8;
        }

        coex_schm_flexible_period_get()
    }

    #[cfg(not(coex))]
    */
    0
}

// 1803
#[allow(unused_variables)]
pub unsafe extern "C" fn coex_register_start_cb(
    cb: Option<unsafe extern "C" fn() -> esp_wifi_sys_esp32c3::c_types::c_int>,
) -> esp_wifi_sys_esp32c3::c_types::c_int {
    //#[cfg(coex)]
    //return unsafe { esp_wifi_sys::include::coex_register_start_cb(_cb) };

    //#[cfg(not(coex))]
    0
}

/// **************************************************************************
/// Name: esp_clk_slowclk_cal_get_wrapper
///
/// Description:
///   Get the calibration value of RTC slow clock
///
/// Input Parameters:
///   None
///
/// Returned Value:
///   The calibration value obtained using rtc_clk_cal
///
/// *************************************************************************
// 1836
pub unsafe extern "C" fn slowclk_cal_get() -> u32 {
    trace!("slowclk_cal_get");

    // TODO not hardcode this

    #[cfg(esp32c3)]
    return 28639;
}
