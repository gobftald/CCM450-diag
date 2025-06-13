#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    t: [T; 1],
    _reserved0: [u8; 0x24],
    wdtconfig0: WDTCONFIG0,
    _reserved1: [u8; 0x18],
    wdtwprotect: WDTWPROTECT,
}

// 26
impl RegisterBlock {
    // 0x00..0x24 - Cluster T%s, containing T?CONFIG, T?LO, T?HI, T?UPDATE, T?ALARMLO, T?ALARMHI, T?LOADLO, T?LOADHI, T?LOAD"]
    #[inline(always)]
    pub const fn t(&self, n: usize) -> &T {
        &self.t[n]
    }

    /// 0x48 - TIMG_WDTCONFIG0_REG.
    #[inline(always)]
    pub const fn wdtconfig0(&self) -> &WDTCONFIG0 {
        &self.wdtconfig0
    }

    /// 0x64 - TIMG_WDTWPROTECT_REG.
    #[inline(always)]
    pub const fn wdtwprotect(&self) -> &WDTWPROTECT {
        &self.wdtwprotect
    }
}

/// Cluster T%s, containing T?CONFIG, T?LO, T?HI, T?UPDATE, T?ALARMLO, T?ALARMHI, T?LOADLO, T?LOADHI, T?LOAD"]
// 125
pub use self::t::T;
pub mod t;

/// WDTCONFIG0 (rw) register accessor: TIMG_WDTCONFIG0_REG.
// 130
pub type WDTCONFIG0 = crate::Reg<wdtconfig0::WDTCONFIG0_SPEC>;
pub mod wdtconfig0;

/// WDTWPROTECT (rw) register accessor: TIMG_WDTWPROTECT_REG.
// 158
pub type WDTWPROTECT = crate::Reg<wdtwprotect::WDTWPROTECT_SPEC>;
pub mod wdtwprotect;
