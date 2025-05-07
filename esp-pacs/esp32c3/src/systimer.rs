#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x04],
    unit_op: [UNIT_OP; 2],
    _reserved1: [u8; 0x34],
    unit_value: [UNIT_VALUE; 2],
}

// 20
impl RegisterBlock {
    /// 0x04..0x0c - SYSTIMER_UNIT%s_OP.
    #[inline(always)]
    // 28
    pub const fn unit_op(&self, n: usize) -> &UNIT_OP {
        &self.unit_op[n]
    }

    /// 0x40..0x50 - Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
    #[inline(always)]
    // 107
    pub const fn unit_value(&self, n: usize) -> &UNIT_VALUE {
        &self.unit_value[n]
    }
}

/// UNIT_OP (rw) register accessor: SYSTIMER_UNIT%s_OP.
// 204
pub type UNIT_OP = crate::Reg<unit_op::UNIT_OP_SPEC>;
pub mod unit_op;

/// Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
// 222
pub use self::unit_value::UNIT_VALUE;
pub mod unit_value;
