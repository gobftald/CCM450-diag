#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    ana_conf0: ANA_CONF0,
}

impl RegisterBlock {
    /// 0x00 - ANA_CONF0 register
    #[inline(always)]
    pub const fn ana_conf0(&self) -> &ANA_CONF0 {
        &self.ana_conf0
    }
}

/// ANA_CONF0 (rw) register accessor: ANA_CONF0 register
// 26
pub type ANA_CONF0 = crate::Reg<ana_conf0::ANA_CONF0_SPEC>;
pub mod ana_conf0;
