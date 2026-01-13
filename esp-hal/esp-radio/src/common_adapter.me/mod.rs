// 1
use esp_wifi_sys::{
    c_types::c_char,
    include::{
        esp_phy_calibration_data_t, esp_phy_calibration_mode_t, register_chipv7_phy, timeval,
    },
};

// 8
//use portable_atomic::{AtomicU32, Ordering};

// 10
use crate::{
    binary::include::{esp_event_base_t, esp_timer_get_time, get_phy_version_str},
    compat::{
        common::{sem_create, sem_delete, sem_give, sem_take, str_from_c},
        timer_compat::{
            compat_timer_arm, compat_timer_arm_us, compat_timer_disarm, compat_timer_done,
            compat_timer_setfn,
        },
    },
    //hal::{self, clock::RadioClockController, peripherals::RADIO_CLK, ram},
    hal::{self, ram},
};

#[cfg_attr(esp32c3, path = "common_adapter_esp32c3.rs")]
// 23
pub(crate) mod chip_specific;

#[cfg_attr(esp32c3, path = "phy_init_data_esp32c3.rs")]
// 32
pub(crate) mod phy_init_data;

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
// 49
pub unsafe extern "C" fn semphr_create(max: u32, init: u32) -> *mut crate::binary::c_types::c_void {
    //trace!("semphr_create - max {} init {}", max, init);
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
// 68
pub unsafe extern "C" fn semphr_delete(semphr: *mut crate::binary::c_types::c_void) {
    //trace!("semphr_delete {:?}", semphr);
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
// 88
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
// 109
pub unsafe extern "C" fn semphr_give(semphr: *mut crate::binary::c_types::c_void) -> i32 {
    sem_give(semphr)
}

/// **************************************************************************
/// Name: esp_random_ulong
/// *************************************************************************
#[ram]
#[unsafe(no_mangle)]
// 119
pub unsafe extern "C" fn random() -> crate::binary::c_types::c_ulong {
    trace!("random");

    // stealing RNG is safe since we own it (passed into `init`)
    //let mut rng = hal::rng::Rng::new(unsafe { hal::peripherals::RNG::steal() });
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
// 141
pub unsafe extern "C" fn read_mac(mac: *mut u8, type_: u32) -> crate::binary::c_types::c_int {
    trace!("read_mac {:?} {}", mac, type_);

    let base_mac = hal::efuse::Efuse::mac_address();

    for (i, &byte) in base_mac.iter().enumerate() {
        unsafe {
            mac.add(i).write_volatile(byte);
        }
    }

    unsafe {
        // ESP_MAC_WIFI_SOFTAP
        if type_ == 1 {
            let tmp = mac.offset(0).read_volatile();
            for i in 0..64 {
                mac.offset(0).write_volatile(tmp | 0x02);
                mac.offset(0)
                    .write_volatile(mac.offset(0).read_volatile() ^ (i << 2));

                if mac.offset(0).read_volatile() != tmp {
                    break;
                }
            }
        }
    }

    0
}

// other functions
#[allow(unused_variables)]
#[unsafe(no_mangle)]
// 210
pub unsafe extern "C" fn puts(s: *const c_char) {
    unsafe {
        let cstr = str_from_c(s);
        info!("{}", cstr);
    }
}

// #define ESP_EVENT_DEFINE_BASE(id) esp_event_base_t id = #id
#[unsafe(no_mangle)]
// 219
static mut WIFI_EVENT: esp_event_base_t = c"WIFI_EVENT".as_ptr();

// stuff needed by wpa-supplicant
#[unsafe(no_mangle)]
// 223
pub unsafe extern "C" fn __assert_func(
    file: *const c_char,
    line: u32,
    func: *const c_char,
    failed_expr: *const c_char,
) {
    unsafe {
        let file = str_from_c(file);
        let (func_pre, func) = if func.is_null() {
            ("", "")
        } else {
            (", function: ", str_from_c(func))
        };
        let expr = str_from_c(failed_expr);

        panic!(
            //"assertion \"{}\" failed: file \"{}\", line {}{}{}",
            "assertion {} failed: file {}, line {}{}{}",
            expr, file, line, func_pre, func
        );
    }
}

#[unsafe(no_mangle)]
// 247
pub unsafe extern "C" fn ets_timer_disarm(timer: *mut crate::binary::c_types::c_void) {
    compat_timer_disarm(timer.cast());
}

#[unsafe(no_mangle)]
// 252
pub unsafe extern "C" fn ets_timer_done(timer: *mut crate::binary::c_types::c_void) {
    compat_timer_done(timer.cast());
}

#[unsafe(no_mangle)]
// 257
pub unsafe extern "C" fn ets_timer_setfn(
    ptimer: *mut crate::binary::c_types::c_void,
    pfunction: *mut crate::binary::c_types::c_void,
    parg: *mut crate::binary::c_types::c_void,
) {
    unsafe {
        compat_timer_setfn(
            ptimer.cast(),
            core::mem::transmute::<
                *mut crate::binary::c_types::c_void,
                unsafe extern "C" fn(*mut crate::binary::c_types::c_void),
            >(pfunction),
            parg,
        );
    }
}

#[unsafe(no_mangle)]
// 275
pub unsafe extern "C" fn ets_timer_arm(
    timer: *mut crate::binary::c_types::c_void,
    tmout: u32,
    repeat: bool,
) {
    compat_timer_arm(timer.cast(), tmout, repeat);
}

#[unsafe(no_mangle)]
// 284
pub unsafe extern "C" fn ets_timer_arm_us(
    timer: *mut crate::binary::c_types::c_void,
    tmout: u32,
    repeat: bool,
) {
    compat_timer_arm_us(timer.cast(), tmout, repeat);
}

#[unsafe(no_mangle)]
// 293
pub unsafe extern "C" fn gettimeofday(tv: *mut timeval, _tz: *mut ()) -> i32 {
    if !tv.is_null() {
        unsafe {
            let microseconds = esp_timer_get_time();
            (*tv).tv_sec = (microseconds / 1_000_000) as u64;
            (*tv).tv_usec = (microseconds % 1_000_000) as u32;
        }
    }

    0
}

#[unsafe(no_mangle)]
// 306
pub unsafe extern "C" fn esp_fill_random(dst: *mut u8, len: u32) {
    trace!("esp_fill_random");
    unsafe {
        let dst = core::slice::from_raw_parts_mut(dst, len as usize);

        // stealing RNG is safe since we own it (passed into `init`)
        //let mut rng = esp_hal::rng::Rng::new(esp_hal::peripherals::RNG::steal());
        let mut rng = esp_hal::rng::Rng::new();
        for chunk in dst.chunks_mut(4) {
            let bytes = rng.random().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

#[unsafe(no_mangle)]
// 321
pub unsafe extern "C" fn strrchr(_s: *const (), _c: u32) -> *const u8 {
    todo!("strrchr");
}

// 325
//static PHY_CLOCK_ENABLE_REF: AtomicU32 = AtomicU32::new(0);

// 327
pub(crate) unsafe fn phy_enable_clock() {
    /*
    let count = PHY_CLOCK_ENABLE_REF.fetch_add(1, Ordering::Acquire);
    if count == 0 {
        // stealing RADIO_CLK is safe since it is passed (as mutable reference or by
        // value) into `init`
        let radio_clocks = unsafe { RADIO_CLK::steal() };
        RadioClockController::new(radio_clocks).enable_phy(true);
        trace!("phy_enable_clock done!");
    }
    */
    // Stealing the peripheral is safe here, as they must have been passed into the relevant
    // initialization functions for the Wi-Fi or BLE controller, if this code gets executed.
    // let clock_guard = unsafe { ModemClockControllerPeripheral::steal() }.enable_phy_clock();
    // core::mem::forget(clock_guard);
}

// 350
#[allow(unused)]
pub(crate) fn phy_calibrate() {
    let mut cal_data: [u8; core::mem::size_of::<esp_phy_calibration_data_t>()] =
        [0u8; core::mem::size_of::<esp_phy_calibration_data_t>()];

    let phy_version = unsafe { get_phy_version_str() };
    trace!("phy_version {}", unsafe { str_from_c(phy_version) });

    let init_data = &phy_init_data::PHY_INIT_DATA_DEFAULT;

    unsafe {
        chip_specific::bbpll_en_usb();

        cfg_if::cfg_if! {
            if #[cfg(phy_full_calibration)] {
                const CALIBRATION_MODE: esp_phy_calibration_mode_t = esp_wifi_sys::include::esp_phy_calibration_mode_t_PHY_RF_CAL_FULL;
            } else {
                const CALIBRATION_MODE: esp_phy_calibration_mode_t = esp_wifi_sys::include::esp_phy_calibration_mode_t_PHY_RF_CAL_PARTIAL;
            }
        };

        /*
        cfg_if::cfg_if! {
            if #[cfg(phy_skip_calibration_after_deep_sleep)] {
                let calibration_mode = if crate::hal::system::reset_reason()
                    == Some(crate::hal::rtc_cntl::SocResetReason::CoreDeepSleep) {
                    esp_wifi_sys::include::esp_phy_calibration_mode_t_PHY_RF_CAL_NONE
                } else {
                    CALIBRATION_MODE
                };
            } else {
        */
        let calibration_mode = CALIBRATION_MODE;
        //    }
        //};

        debug!("Using calibration mode {}", calibration_mode);

        register_chipv7_phy(
            init_data,
            &mut cal_data as *mut _ as *mut crate::binary::include::esp_phy_calibration_data_t,
            calibration_mode,
        );
    }
}
