#[repr(C)]
#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0xf8],
    intr_status_reg_0: INTR_STATUS_REG_0,
    intr_status_reg_1: INTR_STATUS_REG_1,
    _reserved1: [u8; 0x04],
    cpu_int_enable: CPU_INT_ENABLE,
    cpu_int_type: CPU_INT_TYPE,
    cpu_int_clear: CPU_INT_CLEAR,
    _reserved2: [u8; 0x04],
    cpu_int_pri: [CPU_INT_PRI; 32],
    cpu_int_thresh: CPU_INT_THRESH,
}

impl RegisterBlock {
    /// xf8 - mac intr map register
    #[inline(always)]
    pub const fn intr_status_reg_0(&self) -> &INTR_STATUS_REG_0 {
        &self.intr_status_reg_0
    }

    /// 0xfc - mac intr map register
    #[inline(always)]
    pub const fn intr_status_reg_1(&self) -> &INTR_STATUS_REG_1 {
        &self.intr_status_reg_1
    }
    /// 0x104 - mac intr map register
    #[inline(always)]
    pub const fn cpu_int_enable(&self) -> &CPU_INT_ENABLE {
        &self.cpu_int_enable
    }

    /// 0x108 - mac intr map register
    #[inline(always)]
    pub const fn cpu_int_type(&self) -> &CPU_INT_TYPE {
        &self.cpu_int_type
    }

    /// 0x10c - mac intr map register
    #[inline(always)]
    pub const fn cpu_int_clear(&self) -> &CPU_INT_CLEAR {
        &self.cpu_int_clear
    }

    /// 0x114..0x194 - mac intr map register
    #[inline(always)]
    pub const fn cpu_int_pri(&self, n: usize) -> &CPU_INT_PRI {
        &self.cpu_int_pri[n]
    }

    /// 0x194 - mac intr map register
    #[inline(always)]
    pub const fn cpu_int_thresh(&self) -> &CPU_INT_THRESH {
        &self.cpu_int_thresh
    }
}

/// INTR_STATUS_REG_0 (r) register accessor: mac intr map register
pub type INTR_STATUS_REG_0 = crate::Reg<intr_status_reg_0::INTR_STATUS_REG_0_SPEC>;
/// mac intr map register
pub mod intr_status_reg_0;

/// INTR_STATUS_REG_1 (r) register accessor: mac intr map register
pub type INTR_STATUS_REG_1 = crate::Reg<intr_status_reg_1::INTR_STATUS_REG_1_SPEC>;
/// mac intr map register
pub mod intr_status_reg_1;

/// CPU_INT_ENABLE (rw) register accessor: mac intr map register
pub type CPU_INT_ENABLE = crate::Reg<cpu_int_enable::CPU_INT_ENABLE_SPEC>;
/// mac intr map register"]
pub mod cpu_int_enable;

/// CPU_INT_TYPE (rw) register accessor: mac intr map register
pub type CPU_INT_TYPE = crate::Reg<cpu_int_type::CPU_INT_TYPE_SPEC>;
/// mac intr map register
pub mod cpu_int_type;

/// CPU_INT_CLEAR (rw) register accessor: mac intr map register
pub type CPU_INT_CLEAR = crate::Reg<cpu_int_clear::CPU_INT_CLEAR_SPEC>;
/// mac intr map register
pub mod cpu_int_clear;

/// CPU_INT_PRI (rw) register accessor: mac intr map register
pub type CPU_INT_PRI = crate::Reg<cpu_int_pri::CPU_INT_PRI_SPEC>;
/// mac intr map register
pub mod cpu_int_pri;

/// CPU_INT_THRESH (rw) register accessor: mac intr map register
pub type CPU_INT_THRESH = crate::Reg<cpu_int_thresh::CPU_INT_THRESH_SPEC>;
/// mac intr map register
pub mod cpu_int_thresh;
