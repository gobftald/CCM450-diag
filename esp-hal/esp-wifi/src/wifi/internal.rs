// 1
use esp_wifi_sys::include::{
    ESP_WIFI_OS_ADAPTER_MAGIC, ESP_WIFI_OS_ADAPTER_VERSION, WIFI_INIT_CONFIG_MAGIC,
    wifi_init_config_t, wifi_osi_funcs_t, wpa_crypto_funcs_t,
};

// 10
use super::os_adapter::{
    calloc_internal, malloc, malloc_internal, mutex_lock, mutex_unlock, recursive_mutex_create,
    spin_lock_create, task_get_current_task,
};

#[unsafe(no_mangle)]
// 70
static g_wifi_osi_funcs: wifi_osi_funcs_t = wifi_osi_funcs_t {
    _version: ESP_WIFI_OS_ADAPTER_VERSION as i32, // 0
    _env_is_chip: None,                           // 4 Some(env_is_chip),
    _set_intr: None,                              // 8 Some(set_intr),
    _clear_intr: None,                            // 12 Some(clear_intr),
    _set_isr: None,                               // 16 Some(os_adapter_chip_specific::set_isr),
    _ints_on: None,                               // 20 Some(ints_on),
    _ints_off: None,                              // 24 Some(ints_off),
    _is_from_isr: None,                           // 28 Some(is_from_isr),
    _spin_lock_create: Some(spin_lock_create),    // 32
    _spin_lock_delete: None,                      // 36 Some(spin_lock_delete),
    _wifi_int_disable: None,                      // 40 Some(wifi_int_disable),
    _wifi_int_restore: None,                      // 44 Some(wifi_int_restore),
    _task_yield_from_isr: None,                   // 48 Some(task_yield_from_isr),
    _semphr_create: None,                         // 52 Some(semphr_create),
    _semphr_delete: None,                         // 56 Some(semphr_delete),
    _semphr_take: None,                           // 60 Some(semphr_take),
    _semphr_give: None,                           // 64 Some(semphr_give),
    _wifi_thread_semphr_get: None,                // 68 Some(wifi_thread_semphr_get),
    _mutex_create: None,                          // 72 Some(mutex_create),
    _recursive_mutex_create: Some(recursive_mutex_create), // 76
    _mutex_delete: None,                          // 80 Some(mutex_delete),
    _mutex_lock: Some(mutex_lock),                // 84
    _mutex_unlock: Some(mutex_unlock),            // 88
    _queue_create: None,                          // 92 Some(queue_create),
    _queue_delete: None,                          // 96 Some(queue_delete),
    _queue_send: None,                            // 100 Some(queue_send),
    _queue_send_from_isr: None,                   // 104 Some(queue_send_from_isr),
    _queue_send_to_back: None,                    // 108 Some(queue_send_to_back),
    _queue_send_to_front: None,                   // 112 Some(queue_send_to_front),
    _queue_recv: None,                            // 116 Some(queue_recv),
    _queue_msg_waiting: None,                     // 120 Some(queue_msg_waiting),
    _event_group_create: None,                    // 124 Some(event_group_create),
    _event_group_delete: None,                    // 128 Some(event_group_delete),
    _event_group_set_bits: None,                  // 132 Some(event_group_set_bits),
    _event_group_clear_bits: None,                // 136 Some(event_group_clear_bits),
    _event_group_wait_bits: None,                 // 140 Some(event_group_wait_bits),
    _task_create_pinned_to_core: None,            // 144 Some(task_create_pinned_to_core),
    _task_create: None,                           // 148 Some(task_create),
    _task_delete: None,                           // 152 Some(task_delete),
    _task_delay: None,                            // 156 Some(task_delay),
    _task_ms_to_tick: None,                       // 160 Some(task_ms_to_tick),
    _task_get_current_task: Some(task_get_current_task), //164
    _task_get_max_priority: None,                 // 168 Some(task_get_max_priority),
    _malloc: Some(malloc),                        // 172
    _free: None,                                  // 176 Some(free),
    _event_post: None,                            // 180 Some(event_post),
    _get_free_heap_size: None,                    // 184 Some(get_free_heap_size),
    _rand: None,                                  // 188 Some(rand),
    _dport_access_stall_other_cpu_start_wrap: None, // 192 Some(dport_access_stall_other_cpu_start_wrap),
    _dport_access_stall_other_cpu_end_wrap: None, // 196 Some(dport_access_stall_other_cpu_end_wrap),
    _wifi_apb80m_request: None,                   // 200 Some(wifi_apb80m_request),
    _wifi_apb80m_release: None,                   // 204 Some(wifi_apb80m_release),
    _phy_disable: None,                           // 208 Some(phy_disable),
    _phy_enable: None,                            // 212 Some(phy_enable),
    _phy_update_country_info: None,               // 216 Some(phy_update_country_info),
    _read_mac: None,                              // 220 Some(read_mac),
    _timer_arm: None,                             // 224 Some(ets_timer_arm),
    _timer_disarm: None,                          // 228 Some(ets_timer_disarm),
    _timer_done: None,                            // 232 Some(ets_timer_done),
    _timer_setfn: None,                           // 236 Some(ets_timer_setfn),
    _timer_arm_us: None,                          // 240 Some(ets_timer_arm_us),
    _wifi_reset_mac: None,                        // 244 Some(wifi_reset_mac),
    _wifi_clock_enable: None,                     // 248 Some(wifi_clock_enable),
    _wifi_clock_disable: None,                    // 252 Some(wifi_clock_disable),
    _wifi_rtc_enable_iso: None,                   // 256 Some(wifi_rtc_enable_iso),
    _wifi_rtc_disable_iso: None,                  // 260 Some(wifi_rtc_disable_iso),
    _esp_timer_get_time: None,                    // 264 Some(esp_timer_get_time),
    _nvs_set_i8: None,                            // 268 Some(nvs_set_i8),
    _nvs_get_i8: None,                            // 272 Some(nvs_get_i8),
    _nvs_set_u8: None,                            // 276 Some(nvs_set_u8),
    _nvs_get_u8: None,                            // 280 Some(nvs_get_u8),
    _nvs_set_u16: None,                           // 284 Some(nvs_set_u16),
    _nvs_get_u16: None,                           // 288 Some(nvs_get_u16),
    _nvs_open: None,                              // 292 Some(nvs_open),
    _nvs_close: None,                             // 296 Some(nvs_close),
    _nvs_commit: None,                            // 300 Some(nvs_commit),
    _nvs_set_blob: None,                          // 304 Some(nvs_set_blob),
    _nvs_get_blob: None,                          // 308 Some(nvs_get_blob),
    _nvs_erase_key: None,                         // 312 Some(nvs_erase_key),
    _get_random: None,                            // 316 Some(get_random),
    _get_time: None,                              // 320 Some(get_time),
    _random: None,                                // 324 Some(random),

    // experience from an earlier implementation
    // _slowclk_cal_get was inserted here
    #[cfg(any(esp32c3, esp32c2, esp32c6, esp32h2, esp32s3, esp32s2))]
    _slowclk_cal_get: None, // 328 Some(slowclk_cal_get)

    #[cfg(feature = "sys-logs")]
    _log_write: None, // 332 Some(log_write),
    #[cfg(not(feature = "sys-logs"))]
    _log_write: None,
    #[cfg(feature = "sys-logs")]
    _log_writev: None, // 336 Some(log_writev),
    #[cfg(not(feature = "sys-logs"))]
    _log_writev: None,
    _log_timestamp: None,                    // 340 Some(log_timestamp),
    _malloc_internal: Some(malloc_internal), // 344
    _realloc_internal: None,                 // 348 Some(realloc_internal),
    _calloc_internal: Some(calloc_internal), // 352
    _zalloc_internal: None,                  // 356 Some(zalloc_internal),
    _wifi_malloc: None,                      // 360 Some(wifi_malloc),
    _wifi_realloc: None,                     // 364 Some(wifi_realloc),
    _wifi_calloc: None,                      // 368 Some(wifi_calloc),
    _wifi_zalloc: None,                      // 372 Some(wifi_zalloc),
    _wifi_create_queue: None,                // 376 Some(wifi_create_queue),
    _wifi_delete_queue: None,                // 380 Some(wifi_delete_queue),
    _coex_init: None,                        // 384 Some(super::coex_init),
    _coex_deinit: None,                      // 388 Some(coex_deinit),
    _coex_enable: None,                      // 392 Some(coex_enable),
    _coex_disable: None,                     // 396 Some(coex_disable),
    _coex_status_get: None,                  // 400 Some(coex_status_get),
    _coex_condition_set: None,               // 404
    _coex_wifi_request: None,                // 408 Some(coex_wifi_request),
    _coex_wifi_release: None,                // 412 Some(coex_wifi_release),
    _coex_wifi_channel_set: None,            // 416 Some(coex_wifi_channel_set),
    _coex_event_duration_get: None,          // 420 Some(coex_event_duration_get),
    _coex_pti_get: None,                     // 424 Some(coex_pti_get),
    _coex_schm_status_bit_clear: None,       // 428 Some(coex_schm_status_bit_clear),
    _coex_schm_status_bit_set: None,         // 432 Some(coex_schm_status_bit_set),
    _coex_schm_interval_set: None,           // 436 Some(coex_schm_interval_set),
    _coex_schm_interval_get: None,           // 440 Some(coex_schm_interval_get),
    _coex_schm_curr_period_get: None,        // 444 Some(coex_schm_curr_period_get),
    _coex_schm_curr_phase_get: None,         // 448 Some(coex_schm_curr_phase_get),

    /*
    //#[cfg(any(esp32c3, esp32c2, esp32c6, esp32h2, esp32s3, esp32s2))]
    _slowclk_cal_get: None, // 448 Some(slowclk_cal_get),
    */
    //#[cfg(any(esp32, esp32s2))]
    //_phy_common_clock_disable: Some(os_adapter_chip_specific::phy_common_clock_disable),
    //#[cfg(any(esp32, esp32s2))]
    //_phy_common_clock_enable: Some(os_adapter_chip_specific::phy_common_clock_enable),
    _coex_register_start_cb: None, //Some(coex_register_start_cb),

    //#[cfg(esp32c6)]
    //_regdma_link_set_write_wait_content: Some(
    //    os_adapter_chip_specific::regdma_link_set_write_wait_content_dummy,
    //),
    //#[cfg(esp32c6)]
    //_sleep_retention_find_link_by_id: Some(
    //    os_adapter_chip_specific::sleep_retention_find_link_by_id_dummy,
    //),
    _coex_schm_process_restart: None, //Some(coex_schm_process_restart_wrapper),
    _coex_schm_register_cb: None,     //Some(coex_schm_register_cb_wrapper),

    _magic: ESP_WIFI_OS_ADAPTER_MAGIC as i32,

    _coex_schm_flexible_period_set: None, //Some(coex_schm_flexible_period_set),
    _coex_schm_flexible_period_get: None, //Some(coex_schm_flexible_period_get),
};

// 214
const WIFI_ENABLE_WPA3_SAE: u64 = 1 << 0;
const WIFI_ENABLE_ENTERPRISE: u64 = 1 << 7;

// 222
const WIFI_FEATURE_CAPS: u64 = WIFI_ENABLE_WPA3_SAE | WIFI_ENABLE_ENTERPRISE;

#[unsafe(no_mangle)]
// 225
pub(super) static mut g_wifi_feature_caps: u64 = WIFI_FEATURE_CAPS;

// 227
pub(super) static mut G_CONFIG: wifi_init_config_t = wifi_init_config_t {
    osi_funcs: core::ptr::addr_of!(g_wifi_osi_funcs).cast_mut(),

    // dummy for now - populated in init
    wpa_crypto_funcs: wpa_crypto_funcs_t {
        size: 0,
        version: 1,
        aes_wrap: None,
        aes_unwrap: None,
        hmac_sha256_vector: None,
        sha256_prf: None,
        hmac_md5: None,
        hamc_md5_vector: None,
        hmac_sha1: None,
        hmac_sha1_vector: None,
        sha1_prf: None,
        sha1_vector: None,
        pbkdf2_sha1: None,
        rc4_skip: None,
        md5_vector: None,
        aes_encrypt: None,
        aes_encrypt_init: None,
        aes_encrypt_deinit: None,
        aes_decrypt: None,
        aes_decrypt_init: None,
        aes_decrypt_deinit: None,
        aes_128_encrypt: None,
        aes_128_decrypt: None,
        omac1_aes_128: None,
        ccmp_decrypt: None,
        ccmp_encrypt: None,
        aes_gmac: None,
        sha256_vector: None,
        crc32: None,
    },

    static_rx_buf_num: crate::CONFIG.static_rx_buf_num as i32,
    dynamic_rx_buf_num: crate::CONFIG.dynamic_rx_buf_num as i32,
    tx_buf_type: esp_wifi_sys::include::CONFIG_ESP_WIFI_TX_BUFFER_TYPE as i32,
    static_tx_buf_num: crate::CONFIG.static_tx_buf_num as i32,
    dynamic_tx_buf_num: crate::CONFIG.dynamic_tx_buf_num as i32,
    rx_mgmt_buf_type: esp_wifi_sys::include::CONFIG_ESP_WIFI_DYNAMIC_RX_MGMT_BUF as i32,
    rx_mgmt_buf_num: esp_wifi_sys::include::CONFIG_ESP_WIFI_RX_MGMT_BUF_NUM_DEF as i32,
    cache_tx_buf_num: esp_wifi_sys::include::WIFI_CACHE_TX_BUFFER_NUM as i32,
    csi_enable: cfg!(feature = "csi") as i32,
    ampdu_rx_enable: crate::CONFIG.ampdu_rx_enable as i32,
    ampdu_tx_enable: crate::CONFIG.ampdu_tx_enable as i32,
    amsdu_tx_enable: crate::CONFIG.amsdu_tx_enable as i32,
    nvs_enable: 0,
    nano_enable: 0,
    rx_ba_win: crate::CONFIG.rx_ba_win as i32,
    wifi_task_core_id: 0,
    beacon_max_len: esp_wifi_sys::include::WIFI_SOFTAP_BEACON_MAX_LEN as i32,
    mgmt_sbuf_num: esp_wifi_sys::include::WIFI_MGMT_SBUF_NUM as i32,
    feature_caps: WIFI_FEATURE_CAPS,
    sta_disconnected_pm: false,
    espnow_max_encrypt_num: esp_wifi_sys::include::CONFIG_ESP_WIFI_ESPNOW_MAX_ENCRYPT_NUM as i32,
    magic: WIFI_INIT_CONFIG_MAGIC as i32,

    tx_hetb_queue_num: 3,
    dump_hesigb_enable: false,
};
