use esp_wifi_sys::c_types::c_char;

// 13
use crate::{
    binary::include::esp_event_base_t,
    compat::common::{sem_create, sem_delete, sem_give, sem_take, str_from_c},
    hal::ram,
};

// 19
#[cfg_attr(esp32c3, path = "common_adapter_esp32c3.rs")]
pub(crate) mod chip_specific;

/// **************************************************************************
/// Name: esp_semphr_create
///
/// Description:
///   Create and initialize semaphore
///
/// Input Parameters:
///   max  - No mean
///   init - semaphore initialization value
///
/// Returned Value:
///   Semaphore data pointer
///
/// *************************************************************************
#[allow(unused)]
// 52
pub unsafe extern "C" fn semphr_create(max: u32, init: u32) -> *mut crate::binary::c_types::c_void {
    trace!("semphr_create - max {} init {}", max, init);
    sem_create(max, init)
}

/// **************************************************************************
/// Name: esp_semphr_delete
///
/// Description:
///   Delete semaphore
///
/// Input Parameters:
///   semphr - Semaphore data pointer
///
/// Returned Value:
///   None
///
/// *************************************************************************
#[allow(unused)]
// 71
pub unsafe extern "C" fn semphr_delete(semphr: *mut crate::binary::c_types::c_void) {
    trace!("semphr_delete {:?}", semphr);
    sem_delete(semphr);
}

/// **************************************************************************
/// Name: esp_semphr_take
///
/// Description:
///   Wait semaphore within a certain period of time
///
/// Input Parameters:
///   semphr - Semaphore data pointer
///   ticks  - Wait system ticks
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
#[ram]
// 91
pub unsafe extern "C" fn semphr_take(
    semphr: *mut crate::binary::c_types::c_void,
    tick: u32,
) -> i32 {
    sem_take(semphr, tick)
}

/// **************************************************************************
/// Name: esp_semphr_give
///
/// Description:
///   Post semaphore
///
/// Input Parameters:
///   semphr - Semaphore data pointer
///
/// Returned Value:
///   True if success or false if fail
///
/// *************************************************************************
#[ram]
// 112
pub unsafe extern "C" fn semphr_give(semphr: *mut crate::binary::c_types::c_void) -> i32 {
    sem_give(semphr)
}

// other functions
#[unsafe(no_mangle)]
// 213
pub unsafe extern "C" fn puts(s: *const c_char) {
    unsafe {
        let cstr = str_from_c(s);
        info!("{}", cstr);
    }
}

// #define ESP_EVENT_DEFINE_BASE(id) esp_event_base_t id = #id
#[unsafe(no_mangle)]
// 222
static mut WIFI_EVENT: esp_event_base_t = c"WIFI_EVENT".as_ptr();
