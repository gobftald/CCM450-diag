#[repr(C)]
#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x104],
    cpu_int_enable: CPU_INT_ENABLE,
    cpu_int_type: CPU_INT_TYPE,
    _reserved1: [u8; 0x08],
    cpu_int_pri: [CPU_INT_PRI; 32],
    cpu_int_thresh: CPU_INT_THRESH,
}

impl RegisterBlock {
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

/// CPU_INT_ENABLE (rw) register accessor: mac intr map register
pub type CPU_INT_ENABLE = crate::Reg<cpu_int_enable::CPU_INT_ENABLE_SPEC>;
/// mac intr map register"]
pub mod cpu_int_enable;

/// CPU_INT_TYPE (rw) register accessor: mac intr map register
pub type CPU_INT_TYPE = crate::Reg<cpu_int_type::CPU_INT_TYPE_SPEC>;
/// mac intr map register
pub mod cpu_int_type;

/// CPU_INT_PRI (rw) register accessor: mac intr map register
pub type CPU_INT_PRI = crate::Reg<cpu_int_pri::CPU_INT_PRI_SPEC>;
/// mac intr map register
pub mod cpu_int_pri;

/// CPU_INT_THRESH (rw) register accessor: mac intr map register
pub type CPU_INT_THRESH = crate::Reg<cpu_int_thresh::CPU_INT_THRESH_SPEC>;
/// mac intr map register
pub mod cpu_int_thresh;
