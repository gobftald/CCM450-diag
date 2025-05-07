#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
pub struct UNIT_VALUE {
    hi: HI,
    lo: LO,
}

impl UNIT_VALUE {
    /// 0x00 - SYSTIMER_UNIT0_VALUE_HI.
    #[inline(always)]
    pub const fn hi(&self) -> &HI {
        &self.hi
    }
    /// 0x04 - SYSTIMER_UNIT0_VALUE_LO.
    #[inline(always)]
    pub const fn lo(&self) -> &LO {
        &self.lo
    }
}

pub type HI = crate::Reg<hi::HI_SPEC>;
pub mod hi;

pub type LO = crate::Reg<lo::LO_SPEC>;
pub mod lo;
