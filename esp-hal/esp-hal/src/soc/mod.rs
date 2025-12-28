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
