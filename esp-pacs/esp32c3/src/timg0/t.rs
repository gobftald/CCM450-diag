#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Cluster T%s, containing T?CONFIG, T?LO, T?HI, T?UPDATE, T?ALARMLO, T?ALARMHI, T?LOADLO, T?LOADHI, T?LOAD"]
pub struct T {
    config: CONFIG,
}

impl T {
    /// 0x00 - TIMG_T0CONFIG_REG.
    #[inline(always)]
    pub const fn config(&self) -> &CONFIG {
        &self.config
    }
}

// CONFIG (rw) register accessor: TIMG_T0CONFIG_REG.
// 63
pub type CONFIG = crate::Reg<config::CONFIG_SPEC>;
pub mod config;
