#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x04],
    gpio: [GPIO; 22],
}

impl RegisterBlock {
    /// 0x04..0x5c - IO MUX Configure Register for pad XTAL_32K_P
    #[inline(always)]
    pub const fn gpio(&self, n: usize) -> &GPIO {
        &self.gpio[n]
    }
}

/// GPIO (rw) register accessor: IO MUX Configure Register for pad XTAL_32K_P
// 38
pub type GPIO = crate::Reg<gpio::GPIO_SPEC>;
pub mod gpio;
