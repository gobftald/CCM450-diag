#![allow(non_camel_case_types, non_upper_case_globals)]

#[repr(C)]
#[derive(Copy, Clone)]
// 1528
pub struct ets_timer {
    pub next: *mut timer_adpt,
    pub expire: u32,
    pub period: u32,
    pub func: ::core::option::Option<unsafe extern "C" fn(priv_: *mut crate::c_types::c_void)>,
    pub priv_: *mut crate::c_types::c_void,
}

// 3692
pub type esp_err_t = crate::c_types::c_int;

// 7159
pub const wifi_log_level_t_WIFI_LOG_NONE: wifi_log_level_t = 0;
pub const wifi_log_level_t_WIFI_LOG_ERROR: wifi_log_level_t = 1;
pub const wifi_log_level_t_WIFI_LOG_WARNING: wifi_log_level_t = 2;
pub const wifi_log_level_t_WIFI_LOG_INFO: wifi_log_level_t = 3;
pub const wifi_log_level_t_WIFI_LOG_DEBUG: wifi_log_level_t = 4;
pub const wifi_log_level_t_WIFI_LOG_VERBOSE: wifi_log_level_t = 5;
/// @brief WiFi log level\n
// 7166
pub type wifi_log_level_t = crate::c_types::c_uint;

// 7308
unsafe extern "C" {
    /// @brief     Set current WiFi log level
    ///
    /// @param     level   Log level.
    ///
    /// @return
    ///     - ESP_OK: succeed
    ///     - ESP_FAIL: level is invalid
    pub fn esp_wifi_internal_set_log_level(level: wifi_log_level_t) -> esp_err_t;
}

#[repr(C)]
#[derive(Copy, Clone)]
// 9117
pub struct timer_adpt {
    pub _address: u8,
}
