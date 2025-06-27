use esp_wifi_sys::c_types::c_char;

// 13
use crate::{binary::include::esp_event_base_t, compat::common::str_from_c};

// 19
#[cfg_attr(esp32c3, path = "common_adapter_esp32c3.rs")]
pub(crate) mod chip_specific;

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
