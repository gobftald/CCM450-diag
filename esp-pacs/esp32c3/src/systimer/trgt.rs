#[repr(C)]
pub struct TRGT {
    hi: HI,
    lo: LO,
}

impl TRGT {
    /// 0x00 - SYSTIMER_TARGET0_HI.
    #[inline(always)]
    pub const fn hi(&self) -> &HI {
        &self.hi
    }
    /// 0x04 - SYSTIMER_TARGET0_LO.
    #[inline(always)]
    pub const fn lo(&self) -> &LO {
        &self.lo
    }
}

pub type HI = crate::Reg<hi::HI_SPEC>;
pub mod hi;

pub type LO = crate::Reg<lo::LO_SPEC>;
pub mod lo;
