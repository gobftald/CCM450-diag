// 1
use esp_wifi_sys::include::{
    wifi_init_config_t, wifi_osi_funcs_t, wpa_crypto_funcs_t, ESP_WIFI_OS_ADAPTER_MAGIC,
    ESP_WIFI_OS_ADAPTER_VERSION, WIFI_INIT_CONFIG_MAGIC,
};

// 10
use super::os_adapter::{
    calloc_internal, coex_enable, coex_event_duration_get, coex_pti_get, coex_register_start_cb,
    coex_schm_curr_period_get, coex_schm_flexible_period_get, coex_schm_flexible_period_set,
    coex_schm_interval_set, coex_schm_register_cb_wrapper, coex_schm_status_bit_clear,
    coex_schm_status_bit_set, coex_status_get, coex_wifi_channel_set, coex_wifi_release,
    coex_wifi_request, env_is_chip, esp_timer_get_time, event_post, free, get_random, ints_on,
    log_timestamp, malloc, malloc_internal, mutex_delete, mutex_lock, mutex_unlock,
    os_adapter_chip_specific, phy_enable, phy_update_country_info, queue_recv, queue_send,
    queue_send_from_isr, recursive_mutex_create, set_intr, slowclk_cal_get, spin_lock_create,
    spin_lock_delete, task_create_pinned_to_core, task_delay, task_get_current_task,
    task_get_max_priority, task_ms_to_tick, task_yield_from_isr, wifi_apb80m_request, wifi_calloc,
    wifi_clock_enable, wifi_create_queue, wifi_delete_queue, wifi_int_disable, wifi_int_restore,
    wifi_malloc, wifi_reset_mac, wifi_thread_semphr_get, wifi_zalloc, zalloc_internal,
};

#[cfg(feature = "sys-logs")]
use super::os_adapter::{log_write, log_writev};

// 11
use crate::common_adapter::{
    ets_timer_arm, ets_timer_arm_us, ets_timer_disarm, ets_timer_done, ets_timer_setfn, random,
    read_mac, semphr_create, semphr_delete, semphr_give, semphr_take,
};

#[unsafe(no_mangle)]
// 70
static g_wifi_osi_funcs: wifi_osi_funcs_t = wifi_osi_funcs_t {
    _version: ESP_WIFI_OS_ADAPTER_VERSION as i32,      // 0
    _env_is_chip: Some(env_is_chip),                   // 4
    _set_intr: Some(set_intr),                         // 8
    _clear_intr: None,                                 // 12 Some(clear_intr),
    _set_isr: Some(os_adapter_chip_specific::set_isr), // 16
    _ints_on: Some(ints_on),                           // 20
    _ints_off: None,                                   // 24 Some(ints_off),
    _is_from_isr: None,                                // 28 Some(is_from_isr),
    _spin_lock_create: Some(spin_lock_create),         // 32
    _spin_lock_delete: Some(spin_lock_delete),         // 36
    _wifi_int_disable: Some(wifi_int_disable),         // 40
    _wifi_int_restore: Some(wifi_int_restore),         // 44
    _task_yield_from_isr: Some(task_yield_from_isr),   // 48
    _semphr_create: Some(semphr_create),               // 52
    _semphr_delete: Some(semphr_delete),               // 56
    _semphr_take: Some(semphr_take),                   // 60
    _semphr_give: Some(semphr_give),                   // 64
    _wifi_thread_semphr_get: Some(wifi_thread_semphr_get), // 68
    _mutex_create: None,                               // 72 Some(mutex_create),
    _recursive_mutex_create: Some(recursive_mutex_create), // 76
    _mutex_delete: Some(mutex_delete),                 // 80
    _mutex_lock: Some(mutex_lock),                     // 84
    _mutex_unlock: Some(mutex_unlock),                 // 88
    _queue_create: None,                               // 92 Some(queue_create),
    _queue_delete: None,                               // 96 Some(queue_delete),
    _queue_send: Some(queue_send),                     // 100
    _queue_send_from_isr: Some(queue_send_from_isr),   // 104
    _queue_send_to_back: None,                         // 108 Some(queue_send_to_back),
    _queue_send_to_front: None,                        // 112 Some(queue_send_to_front),
    _queue_recv: Some(queue_recv),                     // 116
    _queue_msg_waiting: None,                          // 120 Some(queue_msg_waiting),
    _event_group_create: None,                         // 124 Some(event_group_create),
    _event_group_delete: None,                         // 128 Some(event_group_delete),
    _event_group_set_bits: None,                       // 132 Some(event_group_set_bits),
    _event_group_clear_bits: None,                     // 136 Some(event_group_clear_bits),
    _event_group_wait_bits: None,                      // 140 Some(event_group_wait_bits),
    _task_create_pinned_to_core: Some(task_create_pinned_to_core), // 144
    _task_create: None,                                // 148 Some(task_create),
    _task_delete: None,                                // 152 Some(task_delete),
    _task_delay: Some(task_delay),                     // 156
    _task_ms_to_tick: Some(task_ms_to_tick),           // 160
    _task_get_current_task: Some(task_get_current_task), // 164
    _task_get_max_priority: Some(task_get_max_priority), // 168
    _malloc: Some(malloc),                             // 172
    _free: Some(free),                                 // 176
    _event_post: Some(event_post),                     // 180
    _get_free_heap_size: None,                         // 184 Some(get_free_heap_size),
    _rand: None,                                       // 188 Some(rand),
    _dport_access_stall_other_cpu_start_wrap: None, // 192 Some(dport_access_stall_other_cpu_start_wrap),
    _dport_access_stall_other_cpu_end_wrap: None, // 196 Some(dport_access_stall_other_cpu_end_wrap),
    _wifi_apb80m_request: Some(wifi_apb80m_request), // 200
    _wifi_apb80m_release: None,                   // 204 Some(wifi_apb80m_release),
    _phy_disable: None,                           // 208 Some(phy_disable),
    _phy_enable: Some(phy_enable),                // 212
    _phy_update_country_info: Some(phy_update_country_info), // 216
    _read_mac: Some(read_mac),                    // 220
    _timer_arm: Some(ets_timer_arm),              // 224
    _timer_disarm: Some(ets_timer_disarm),        // 228
    _timer_done: Some(ets_timer_done),            // 232
    _timer_setfn: Some(ets_timer_setfn),          // 236
    _timer_arm_us: Some(ets_timer_arm_us),        // 240
    _wifi_reset_mac: Some(wifi_reset_mac),        // 244
    _wifi_clock_enable: Some(wifi_clock_enable),  // 248
    _wifi_clock_disable: None,                    // 252 Some(wifi_clock_disable),
    _wifi_rtc_enable_iso: None,                   // 256 Some(wifi_rtc_enable_iso),
    _wifi_rtc_disable_iso: None,                  // 260 Some(wifi_rtc_disable_iso),
    _esp_timer_get_time: Some(esp_timer_get_time), // 264
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
    _get_random: Some(get_random),                // 316
    _get_time: None,                              // 320 Some(get_time),
    _random: Some(random),                        // 324

    //here is the _slowclk_cal_get position
    //
    #[cfg(feature = "sys-logs")]
    _log_write: Some(log_write), // 332
    #[cfg(not(feature = "sys-logs"))]
    _log_write: None,
    #[cfg(feature = "sys-logs")]
    _log_writev: Some(log_writev), // 336
    #[cfg(not(feature = "sys-logs"))]
    _log_writev: None,
    _log_timestamp: Some(log_timestamp),                 // 340
    _malloc_internal: Some(malloc_internal),             // 344
    _realloc_internal: None,                             // 348 Some(realloc_internal),
    _calloc_internal: Some(calloc_internal),             // 352
    _zalloc_internal: Some(zalloc_internal),             // 356
    _wifi_malloc: Some(wifi_malloc),                     // 360
    _wifi_realloc: None,                                 // 364 Some(wifi_realloc),
    _wifi_calloc: Some(wifi_calloc),                     // 368
    _wifi_zalloc: Some(wifi_zalloc),                     // 372
    _wifi_create_queue: Some(wifi_create_queue),         // 376
    _wifi_delete_queue: Some(wifi_delete_queue),         // 380
    _coex_init: None,                                    // 384 Some(super::coex_init),
    _coex_deinit: None,                                  // 388 Some(coex_deinit),
    _coex_enable: Some(coex_enable),                     // 392
    _coex_disable: None,                                 // 396 Some(coex_disable),
    _coex_status_get: Some(coex_status_get),             // 400
    _coex_condition_set: None,                           // 404
    _coex_wifi_request: Some(coex_wifi_request),         // 408
    _coex_wifi_release: Some(coex_wifi_release),         // 412
    _coex_wifi_channel_set: Some(coex_wifi_channel_set), //416
    _coex_event_duration_get: Some(coex_event_duration_get), // 420
    _coex_pti_get: Some(coex_pti_get),                   // 424
    _coex_schm_status_bit_clear: Some(coex_schm_status_bit_clear), // 428
    _coex_schm_status_bit_set: Some(coex_schm_status_bit_set), // 432
    _coex_schm_interval_set: Some(coex_schm_interval_set), // 436
    _coex_schm_interval_get: None,                       // 440 Some(coex_schm_interval_get),
    _coex_schm_curr_period_get: Some(coex_schm_curr_period_get), // 444
    _coex_schm_curr_phase_get: None,                     // 448 Some(coex_schm_curr_phase_get),

    #[cfg(any(esp32c3, esp32c2, esp32c6, esp32h2, esp32s3, esp32s2))]
    _slowclk_cal_get: Some(slowclk_cal_get), // 328

    #[cfg(any(esp32, esp32s2))]
    _phy_common_clock_disable: Some(os_adapter_chip_specific::phy_common_clock_disable),
    #[cfg(any(esp32, esp32s2))]
    _phy_common_clock_enable: Some(os_adapter_chip_specific::phy_common_clock_enable),
    _coex_register_start_cb: Some(coex_register_start_cb), // 460

    #[cfg(esp32c6)]
    _regdma_link_set_write_wait_content: Some(
        os_adapter_chip_specific::regdma_link_set_write_wait_content_dummy,
    ),
    #[cfg(esp32c6)]
    _sleep_retention_find_link_by_id: Some(
        os_adapter_chip_specific::sleep_retention_find_link_by_id_dummy,
    ),
    _coex_schm_process_restart: None, // 452 Some(coex_schm_process_restart_wrapper),
    _coex_schm_register_cb: Some(coex_schm_register_cb_wrapper), // 456

    _magic: ESP_WIFI_OS_ADAPTER_MAGIC as i32, // 472

    _coex_schm_flexible_period_set: Some(coex_schm_flexible_period_set), // 464
    _coex_schm_flexible_period_get: Some(coex_schm_flexible_period_get), // 468
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
