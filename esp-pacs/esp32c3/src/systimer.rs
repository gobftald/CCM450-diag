#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x04],
    unit_op: [UNIT_OP; 2],
    _reserved1: [u8; 0x34],
    unit_value: [UNIT_VALUE; 2],
}

impl RegisterBlock {
    /// 0x04..0x0c - SYSTIMER_UNIT%s_OP.
    #[inline(always)]
    pub const fn unit_op(&self, n: usize) -> &UNIT_OP {
        &self.unit_op[n]
    }

    /// 0x40..0x50 - Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
    #[inline(always)]
    pub const fn unit_value(&self, n: usize) -> &UNIT_VALUE {
        &self.unit_value[n]
    }
}

/// UNIT_OP (rw) register accessor: SYSTIMER_UNIT%s_OP.
///
/// You can [`read`](crate::Reg::read) this register and get [`unit_op::R`].
/// You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write),
/// [`write_with_zero`](crate::Reg::write_with_zero) this register using [`unit_op::W`].
/// You can also [`modify`](crate::Reg::modify) this register.
/// See [API](https://docs.rs/svd2rust/#read--modify--write-api).
///
/// For information about available fields see [`mod@unit_op`] module"]
pub type UNIT_OP = crate::Reg<unit_op::UNIT_OP_SPEC>;
/// SYSTIMER_UNIT%s_OP.
pub mod unit_op;

/// Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
pub use self::unit_value::UNIT_VALUE;
/// Cluster
/// Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
pub mod unit_value;
