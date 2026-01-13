// 5
pub use self::implementation::*;

#[cfg_attr(esp32c3, path = "esp32c3/mod.rs")]
mod implementation;

// 16
mod efuse_field;

// Indicates the state of setting the mac address
// 0 -- unset
// 1 -- in the process of being set
// 2 -- set
//
// Values other than 0 indicate that we cannot attempt setting the mac address
// again, and values other than 2 indicate that we should read the mac address
// from eFuse.
//static MAC_OVERRIDE_STATE: AtomicU8 = AtomicU8::new(0);
// we don't want to complicate this stuff
// 63
static mut MAC_OVERRIDE_STATE: u8 = 0;
// 65
static mut MAC_OVERRIDE: [u8; 6] = [0; 6];

// 76
impl self::efuse::Efuse {
    /// Get base mac address
    ///
    /// By default this reads the base mac address from eFuse, but it can be
    /// overridden by `set_mac_address`.
    // 105
    pub fn mac_address() -> [u8; 6] {
        //if MAC_OVERRIDE_STATE.load(Ordering::Relaxed) == 2 {
        unsafe {
            if MAC_OVERRIDE_STATE == 2 {
                MAC_OVERRIDE
            } else {
                Self::read_base_mac_address()
            }
        }
    }
}
unsafe extern "C" {
    static mut __stack_chk_guard: u32;
}

#[cfg(all(feature = "rt", riscv))]
fn setup_stack_guard() {
    unsafe extern "C" {
        static mut __stack_chk_guard: u32;
    }

    unsafe {
        let stack_chk_guard = core::ptr::addr_of_mut!(__stack_chk_guard);
        // we _should_ use a random value but we don't have a good source for random
        // numbers here
        stack_chk_guard.write_volatile(esp_config::esp_config_int!(
            u32,
            "ESP_HAL_CONFIG_STACK_GUARD_VALUE"
        ));
    }
}

#[cfg(riscv)]
#[unsafe(export_name = "hal_main")]
// 525
fn hal_main(a0: usize, a1: usize, a2: usize) -> ! {
    unsafe extern "Rust" {
        // This symbol will be provided by the user via `#[entry]`
        fn main(a0: usize, a1: usize, a2: usize) -> !;
    }

    setup_stack_guard();

    unsafe {
        main(a0, a1, a2);
    }
}
