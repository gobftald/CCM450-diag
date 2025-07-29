#![no_std]
#![cfg_attr(feature = "sys-logs", feature(c_variadic))]

#[allow(unused_imports)]
#[macro_use(info, unwrap)]
extern crate console;

// 5
pub mod c_types;

// 11
#[cfg_attr(feature = "esp32c3", path = "include/esp32c3.rs")]
pub mod include;

#[cfg(feature = "sys-logs")]
// 19
pub mod log {
    #[unsafe(no_mangle)]
    // 26
    pub unsafe extern "C" gd (s: *const u8, args: ...) {
        unsafe {
            syslog(0, s, args);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn net80211_printf(s: *const u8, args: ...) {
        unsafe {
            syslog(0, s, args);
        }
    }

    // 41
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn pp_printf(s: *const u8, args: ...) {
        unsafe {
            syslog(0, s, args);
        }
    }

    // 46
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn syslog(
        _priority: u32,
        format: *const u8,
        args: core::ffi::VaListImpl,
    ) {
        //#[allow(clashing_extern_declarations)]
        unsafe extern "C" {
            fn vsnprintf(buffer: *mut u8, len: usize, fmt: *const u8, args: ...);
        }

        let mut buf = [0u8; 512];
        let _res_str;
        unsafe {
            vsnprintf(&mut buf as *mut u8, 512, format, args);
            let res_cstr = core::ffi::CStr::from_ptr(&buf as *const _ as *const core::ffi::c_char);
            _res_str = res_cstr
                .to_str()
                .unwrap_or_else(|_err| core::str::from_utf8_unchecked(br"Utf8Error"));
        }
        //info!("{}", res_str.to_str().unwrap());
        info!("{}", _res_str);
    }
}

#[cfg(not(feature = "sys-logs"))]
// 64
pub mod log {
    #[cfg(target_arch = "riscv32")]
    type VaargType = *const ();

    #[unsafe(no_mangle)]
    // 77
    pub unsafe extern "C" fn phy_printf(_s: *const u8, _args: VaargType) {}

    #[unsafe(no_mangle)]
    // 83
    pub unsafe extern "C" fn net80211_printf(_s: *const u8, _args: VaargType) {}

    #[unsafe(no_mangle)]
    // 86
    pub unsafe extern "C" fn pp_printf(_s: *const u8, _args: VaargType) {}
}
