use crate::hal::peripherals::{APB_CTRL, LPWR};

use crate::{
    compat::common::*,
    hal::{self, ram},
    sys::c_types::{c_char, c_int, c_ulong, c_void},
    sys::include::{esp_event_base_t, timeval},
    time::blob_ticks_to_micros,
};

// 16
pub(crate) fn enable_wifi_power_domain() {
    const SYSTEM_WIFIBB_RST: u32 = 1 << 0;
    const SYSTEM_FE_RST: u32 = 1 << 1;
    const SYSTEM_WIFIMAC_RST: u32 = 1 << 2;
    const SYSTEM_BTBB_RST: u32 = 1 << 3; // Bluetooth Baseband
    const SYSTEM_BTMAC_RST: u32 = 1 << 4; // deprecated
    const SYSTEM_RW_BTMAC_RST: u32 = 1 << 9; // Bluetooth MAC
    const SYSTEM_RW_BTMAC_REG_RST: u32 = 1 << 11; // Bluetooth MAC Regsiters
    const SYSTEM_BTBB_REG_RST: u32 = 1 << 13; // Bluetooth Baseband Registers

    const MODEM_RESET_FIELD_WHEN_PU: u32 = SYSTEM_WIFIBB_RST
        | SYSTEM_FE_RST
        | SYSTEM_WIFIMAC_RST
        | SYSTEM_BTBB_RST
        | SYSTEM_BTMAC_RST
        | SYSTEM_RW_BTMAC_RST
        | SYSTEM_RW_BTMAC_REG_RST
        | SYSTEM_BTBB_REG_RST;

    LPWR::regs()
        .dig_pwc()
        .modify(|_, w| w.wifi_force_pd().clear_bit());

    APB_CTRL::regs()
        .wifi_rst_en()
        .modify(|r, w| unsafe { w.bits(r.bits() | MODEM_RESET_FIELD_WHEN_PU) });

    APB_CTRL::regs()
        .wifi_rst_en()
        .modify(|r, w| unsafe { w.bits(r.bits() & !MODEM_RESET_FIELD_WHEN_PU) });

    LPWR::regs()
        .dig_iso()
        .modify(|_, w| w.wifi_force_iso().clear_bit());
}

#[cfg(feature = "wifi")]
pub unsafe extern "C" fn ets_timer_arm(timer: *mut c_void, ms: u32, repeat: bool) {
    crate::compat::timer_compat::compat_timer_arm(timer.cast(), ms, repeat);
}

#[cfg(feature = "wifi")]
pub unsafe extern "C" fn ets_timer_arm_us(timer: *mut c_void, us: u32, repeat: bool) {
    crate::compat::timer_compat::compat_timer_arm_us(timer.cast(), us, repeat);
}

#[cfg(feature = "wifi")]
pub unsafe extern "C" fn ets_timer_disarm(timer: *mut c_void) {
    crate::compat::timer_compat::compat_timer_disarm(timer.cast());
}

#[cfg(feature = "wifi")]
pub unsafe extern "C" fn ets_timer_done(timer: *mut c_void) {
    crate::compat::timer_compat::compat_timer_done(timer.cast());
}

#[cfg(feature = "wifi")]
pub unsafe extern "C" fn ets_timer_setfn(
    ptimer: *mut c_void,
    pfunction: *mut c_void,
    parg: *mut c_void,
) {
    unsafe {
        crate::compat::timer_compat::compat_timer_setfn(
            ptimer.cast(),
            core::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut c_void)>(pfunction),
            parg,
        );
    }
}

/// **************************************************************************
/// Name: esp_random_ulong
/// *************************************************************************
#[allow(unused)]
#[ram]
pub unsafe extern "C" fn random() -> c_ulong {
    trace!("random");

    let mut rng = hal::rng::Rng::new();
    rng.random()
}

/// **************************************************************************
/// Name: esp_wifi_read_mac
///
/// Description:
///   Read MAC address from efuse
///
/// Input Parameters:
///   mac  - MAC address buffer pointer
///   type - MAC address type
///
/// Returned Value:
///   0 if success or -1 if fail
///
/// *************************************************************************
pub unsafe extern "C" fn read_mac(mac: *mut u8, type_: u32) -> c_int {
    trace!("read_mac {:?} {}", mac, type_);

    let base_mac = hal::efuse::Efuse::mac_address();

    for (i, &byte) in base_mac.iter().enumerate() {
        unsafe {
            mac.add(i).write_volatile(byte);
        }
    }

    const ESP_MAC_WIFI_SOFTAP: u32 = 1;
    const ESP_MAC_BT: u32 = 2;

    unsafe {
        if type_ == ESP_MAC_WIFI_SOFTAP {
            let tmp = mac.offset(0).read_volatile();
            for i in 0..64 {
                mac.offset(0).write_volatile(tmp | 0x02);
                mac.offset(0)
                    .write_volatile(mac.offset(0).read_volatile() ^ (i << 2));

                if mac.offset(0).read_volatile() != tmp {
                    break;
                }
            }
        } else if type_ == ESP_MAC_BT {
            let tmp = mac.offset(0).read_volatile();
            for i in 0..64 {
                mac.offset(0).write_volatile(tmp | 0x02);
                mac.offset(0)
                    .write_volatile(mac.offset(0).read_volatile() ^ (i << 2));

                if mac.offset(0).read_volatile() != tmp {
                    break;
                }
            }

            mac.offset(5)
                .write_volatile(mac.offset(5).read_volatile() + 1);
        } else {
            return -1;
        }
    }

    0
}

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
pub unsafe extern "C" fn semphr_create(max: u32, init: u32) -> *mut c_void {
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
pub unsafe extern "C" fn semphr_delete(semphr: *mut c_void) {
    trace!("semphr_delete {:?}", semphr);
    sem_delete(semphr);
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
pub unsafe extern "C" fn semphr_give(semphr: *mut c_void) -> i32 {
    trace!(">>>> semphr_give {:?}", semphr);
    sem_give(semphr)
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
pub unsafe extern "C" fn semphr_take(semphr: *mut c_void, tick: u32) -> i32 {
    trace!(">>>> semphr_take {:?} block_time_tick {}", semphr, tick);
    sem_take(semphr, blob_ticks_to_micros(tick))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __esp_radio_esp_fill_random(dst: *mut u8, len: u32) {
    trace!("esp_fill_random");
    unsafe {
        let dst = core::slice::from_raw_parts_mut(dst, len as usize);

        let mut rng = esp_hal::rng::Rng::new();
        for chunk in dst.chunks_mut(4) {
            let bytes = rng.random().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

// other functions
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __esp_radio_puts(s: *const c_char) {
    unsafe {
        let cstr = str_from_c(s);
        info!("{}", cstr);
    }
}

// #define ESP_EVENT_DEFINE_BASE(id) esp_event_base_t id = #id
#[unsafe(no_mangle)]
static mut __ESP_RADIO_WIFI_EVENT: esp_event_base_t = c"WIFI_EVENT".as_ptr();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __esp_radio_gettimeofday(tv: *mut timeval, _tz: *mut ()) -> i32 {
    if !tv.is_null() {
        unsafe {
            let microseconds = __esp_radio_esp_timer_get_time();
            (*tv).tv_sec = (microseconds / 1_000_000) as u64;
            (*tv).tv_usec = (microseconds % 1_000_000) as u32;
        }
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __esp_radio_esp_timer_get_time() -> i64 {
    trace!("esp_timer_get_time");
    // Just using IEEE802.15.4 doesn't need the current time. If we don't use `preempt::now`, users
    // will not need to have a scheduler in their firmware.
    cfg_if::cfg_if! {
        if #[cfg(feature = "wifi")] {
            crate::preempt::now() as i64
        } else {
            // In this case we don't have a scheduler, we can return esp-hal's timestamp.
            esp_hal::time::Instant::now().duration_since_epoch().as_micros() as i64
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __esp_radio_strrchr(_s: *const (), _c: u32) -> *const u8 {
    todo!("strrchr");
}

#[allow(unused)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __esp_radio_esp_event_post(
    event_base: *const c_char,
    event_id: i32,
    event_data: *mut c_void,
    event_data_size: usize,
    ticks_to_wait: u32,
) -> i32 {
    #[cfg(feature = "wifi")]
    return unsafe {
        crate::wifi::event_post(
            event_base,
            event_id,
            event_data,
            event_data_size,
            ticks_to_wait,
        )
    };

    #[cfg(not(feature = "wifi"))]
    return -1;
}
