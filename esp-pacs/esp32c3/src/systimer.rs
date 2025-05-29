#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    conf: CONF,
    unit_op: [UNIT_OP; 2],
    _reserved0: [u8; 0x10],
    trgt: [TRGT; 3],
    target_conf: [TARGET_CONF; 3],
    unit_value: [UNIT_VALUE; 2],
    comp_load: [COMP_LOAD; 3],
    _reserved1: [u8; 0x08],
    int_ena: INT_ENA,
    _reserved2: [u8; 0x04],
    int_clr: INT_CLR,
}

// 20
impl RegisterBlock {
    /// 0x00 - SYSTIMER_CONF.
    #[inline(always)]
    pub const fn conf(&self) -> &CONF {
        &self.conf
    }

    /// 0x04..0x0c - SYSTIMER_UNIT%s_OP.
    #[inline(always)]
    // 28
    pub const fn unit_op(&self, n: usize) -> &UNIT_OP {
        &self.unit_op[n]
    }

    /// 0x1c..0x34 - Cluster TRGT%s, containing TARGET?_HI, TARGET?_LO
    #[inline(always)]
    pub const fn trgt(&self, n: usize) -> &TRGT {
        &self.trgt[n]
    }

    /// 0x34..0x40 - SYSTIMER_TARGET%s_CONF.
    #[inline(always)]
    pub const fn target_conf(&self, n: usize) -> &TARGET_CONF {
        &self.target_conf[n]
    }

    /// 0x40..0x50 - Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
    #[inline(always)]
    // 107
    pub const fn unit_value(&self, n: usize) -> &UNIT_VALUE {
        &self.unit_value[n]
    }

    /// 0x50..0x5c - SYSTIMER_COMP%s_LOAD.
    #[inline(always)]
    // 128
    pub const fn comp_load(&self, n: usize) -> &COMP_LOAD {
        &self.comp_load[n]
    }

    /// 0x64 - SYSTIMER_INT_ENA.
    #[inline(always)]
    // 175
    pub const fn int_ena(&self) -> &INT_ENA {
        &self.int_ena
    }

    /// 0x6c - SYSTIMER_INT_CLR.
    #[inline(always)]
    // 185
    pub const fn int_clr(&self) -> &INT_CLR {
        &self.int_clr
    }
}

/// CONF (rw) register accessor: SYSTIMER_CONF.
// 200
pub type CONF = crate::Reg<conf::CONF_SPEC>;
pub mod conf;

/// UNIT_OP (rw) register accessor: SYSTIMER_UNIT%s_OP.
// 204
pub type UNIT_OP = crate::Reg<unit_op::UNIT_OP_SPEC>;
pub mod unit_op;

/// Cluster TRGT%s, containing TARGET?_HI, TARGET?_LO
pub use self::trgt::TRGT;
pub mod trgt;

/// TARGET_CONF (rw) register accessor: SYSTIMER_TARGET%s_CONF.
// 218
pub type TARGET_CONF = crate::Reg<target_conf::TARGET_CONF_SPEC>;
pub mod target_conf;

/// Cluster UNIT%s_VALUE, containing UNIT?_VALUE_HI, UNIT?_VALUE_LO
// 222
pub use self::unit_value::UNIT_VALUE;
pub mod unit_value;

/// COMP_LOAD (w) register accessor: SYSTIMER_COMP%s_LOAD.
// 227
pub type COMP_LOAD = crate::Reg<comp_load::COMP_LOAD_SPEC>;
pub mod comp_load;

/// INT_ENA (rw) register accessor: SYSTIMER_INT_ENA.
// 235
pub type INT_ENA = crate::Reg<int_ena::INT_ENA_SPEC>;
pub mod int_ena;

/// INT_CLR (w) register accessor: SYSTIMER_INT_CLR.
// 243
pub type INT_CLR = crate::Reg<int_clr::INT_CLR_SPEC>;
pub mod int_clr;
