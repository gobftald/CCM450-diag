#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x0c],
    int_ena: INT_ENA,
    int_clr: INT_CLR,
    clkdiv: CLKDIV,
    _reserved1: [u8; 0x08],
    conf0: CONF0,
    conf1: CONF1,
    _reserved2: [u8; 0x20],
    idle_conf: IDLE_CONF,
    _reserved3: [u8; 0x14],
    mem_conf: MEM_CONF,
    _reserved4: [u8; 0x14],
    clk_conf: CLK_CONF,
    _reserved: [u8; 0x04],
    id: ID,
}

impl RegisterBlock {
    /// 0x0c - Interrupt enable bits
    #[inline(always)]
    pub const fn int_ena(&self) -> &INT_ENA {
        &self.int_ena
    }

    /// 0x10 - Interrupt clear bits
    #[inline(always)]
    pub const fn int_clr(&self) -> &INT_CLR {
        &self.int_clr
    }

    /// 0x14 - Clock divider configuration
    #[inline(always)]
    pub const fn clkdiv(&self) -> &CLKDIV {
        &self.clkdiv
    }

    // 0x20 - a
    #[inline(always)]
    pub const fn conf0(&self) -> &CONF0 {
        &self.conf0
    }

    /// 0x24 - Configuration register 1
    #[inline(always)]
    pub const fn conf1(&self) -> &CONF1 {
        &self.conf1
    }

    /// 0x48 - Frame-end idle configuration"]
    #[inline(always)]
    pub const fn idle_conf(&self) -> &IDLE_CONF {
        &self.idle_conf
    }

    /// 0x60 - UART threshold and allocation configuration
    #[inline(always)]
    pub const fn mem_conf(&self) -> &MEM_CONF {
        &self.mem_conf
    }

    /// 0x78 - UART core clock configuration
    #[inline(always)]
    pub const fn clk_conf(&self) -> &CLK_CONF {
        &self.clk_conf
    }

    /// 0x80 - UART ID register
    #[inline(always)]
    pub const fn id(&self) -> &ID {
        &self.id
    }
}

/// INT_ENA (rw) register accessor: Interrupt enable bits
// 219
pub type INT_ENA = crate::Reg<int_ena::INT_ENA_SPEC>;
pub mod int_ena;

/// INT_CLR (w) register accessor: Interrupt clear bits
// 223
pub type INT_CLR = crate::Reg<int_clr::INT_CLR_SPEC>;
pub mod int_clr;

/// CLKDIV (rw) register accessor: Clock divider configuration
// 227
pub type CLKDIV = crate::Reg<clkdiv::CLKDIV_SPEC>;
pub mod clkdiv;

/// CONF0 (rw) register accessor: a
// 239
pub type CONF0 = crate::Reg<conf0::CONF0_SPEC>;
pub mod conf0;

/// CONF1 (rw) register accessor: Configuration register 1
// 243
pub type CONF1 = crate::Reg<conf1::CONF1_SPEC>;
pub mod conf1;

/// IDLE_CONF (rw) register accessor: Frame-end idle configuration
pub type IDLE_CONF = crate::Reg<idle_conf::IDLE_CONF_SPEC>;
pub mod idle_conf;

/// MEM_CONF (rw) register accessor: UART threshold and allocation configuration
// 303
pub type MEM_CONF = crate::Reg<mem_conf::MEM_CONF_SPEC>;
pub mod mem_conf;

/// CLK_CONF (rw) register accessor: UART core clock configuration
// 327
pub type CLK_CONF = crate::Reg<clk_conf::CLK_CONF_SPEC>;
pub mod clk_conf;

/// ID (rw) register accessor: UART ID register
// 335
pub type ID = crate::Reg<id::ID_SPEC>;
pub mod id;
