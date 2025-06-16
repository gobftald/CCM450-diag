#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Cluster T%s, containing T?CONFIG, T?LO, T?HI, T?UPDATE, T?ALARMLO, T?ALARMHI, T?LOADLO, T?LOADHI, T?LOAD"]
pub struct T {
    config: CONFIG,
    _reserved0: [u8; 0x0c],
    alarmlo: ALARMLO,
    alarmhi: ALARMHI,
    loadlo: LOADLO,
    loadhi: LOADHI,
    load: LOAD,
}

impl T {
    /// 0x00 - TIMG_T0CONFIG_REG.
    #[inline(always)]
    pub const fn config(&self) -> &CONFIG {
        &self.config
    }

    /// 0x10 - TIMG_T0ALARMLO_REG.
    #[inline(always)]
    pub const fn alarmlo(&self) -> &ALARMLO {
        &self.alarmlo
    }

    /// 0x14 - TIMG_T0ALARMHI_REG.
    #[inline(always)]
    pub const fn alarmhi(&self) -> &ALARMHI {
        &self.alarmhi
    }

    /// 0x18 - TIMG_T0LOADLO_REG.
    #[inline(always)]
    pub const fn loadlo(&self) -> &LOADLO {
        &self.loadlo
    }

    /// 0x1c - TIMG_T0LOADHI_REG.
    #[inline(always)]
    pub const fn loadhi(&self) -> &LOADHI {
        &self.loadhi
    }

    /// 0x20 - TIMG_T0LOAD_REG.
    #[inline(always)]
    pub const fn load(&self) -> &LOAD {
        &self.load
    }
}

/// CONFIG (rw) register accessor: TIMG_T0CONFIG_REG.
// 63
pub type CONFIG = crate::Reg<config::CONFIG_SPEC>;
pub mod config;

/// ALARMLO (rw) register accessor: TIMG_T0ALARMLO_REG.
// 79
pub type ALARMLO = crate::Reg<alarmlo::ALARMLO_SPEC>;
pub mod alarmlo;

/// ALARMHI (rw) register accessor: TIMG_T0ALARMHI_REG.
pub type ALARMHI = crate::Reg<alarmhi::ALARMHI_SPEC>;
pub mod alarmhi;

/// LOADLO (rw) register accessor: TIMG_T0LOADLO_REG.
pub type LOADLO = crate::Reg<loadlo::LOADLO_SPEC>;
pub mod loadlo;

/// LOADHI (rw) register accessor: TIMG_T0LOADHI_REG.
pub type LOADHI = crate::Reg<loadhi::LOADHI_SPEC>;
pub mod loadhi;

/// LOAD (w) register accessor: TIMG_T0LOAD_REG.
pub type LOAD = crate::Reg<load::LOAD_SPEC>;
pub mod load;
