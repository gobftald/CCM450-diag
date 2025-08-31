#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    fifo: FIFO,
    int_raw: INT_RAW,
    int_st: INT_ST,
    int_ena: INT_ENA,
    int_clr: INT_CLR,
    clkdiv: CLKDIV,
    _reserved0: [u8; 0x04],
    status: STATUS,
    conf0: CONF0,
    conf1: CONF1,
    _reserved1: [u8; 0x20],
    idle_conf: IDLE_CONF,
    _reserved2: [u8; 0x4],
    at_cmd_precnt: AT_CMD_PRECNT,
    at_cmd_postcnt: AT_CMD_POSTCNT,
    at_cmd_gaptout: AT_CMD_GAPTOUT,
    at_cmd_char: AT_CMD_CHAR,
    mem_conf: MEM_CONF,
    _reserved3: [u8; 0x08],
    fsm_status: FSM_STATUS,
    _reserved4: [u8; 0x08],
    clk_conf: CLK_CONF,
    _reserved5: [u8; 0x04],
    id: ID,
}

impl RegisterBlock {
    /// 0x00 - FIFO data register"]
    #[inline(always)]
    pub const fn fifo(&self) -> &FIFO {
        &self.fifo
    }

    /// 0x04 - Raw interrupt status
    #[inline(always)]
    pub const fn int_raw(&self) -> &INT_RAW {
        &self.int_raw
    }

    /// 0x08 - Masked interrupt status
    #[inline(always)]
    pub const fn int_st(&self) -> &INT_ST {
        &self.int_st
    }

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

    /// 0x1c - UART status register
    #[inline(always)]
    pub const fn status(&self) -> &STATUS {
        &self.status
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

    /// 0x50 - Pre-sequence timing configuration
    #[inline(always)]
    pub const fn at_cmd_precnt(&self) -> &AT_CMD_PRECNT {
        &self.at_cmd_precnt
    }
    /// 0x54 - Post-sequence timing configuration
    #[inline(always)]
    pub const fn at_cmd_postcnt(&self) -> &AT_CMD_POSTCNT {
        &self.at_cmd_postcnt
    }
    /// 0x58 - Timeout configuration
    #[inline(always)]
    pub const fn at_cmd_gaptout(&self) -> &AT_CMD_GAPTOUT {
        &self.at_cmd_gaptout
    }

    /// 0x5c - AT escape sequence detection configuration
    #[inline(always)]
    pub const fn at_cmd_char(&self) -> &AT_CMD_CHAR {
        &self.at_cmd_char
    }

    /// 0x60 - UART threshold and allocation configuration
    #[inline(always)]
    pub const fn mem_conf(&self) -> &MEM_CONF {
        &self.mem_conf
    }

    /// 0x6c - UART transmit and receive status.
    #[inline(always)]
    pub const fn fsm_status(&self) -> &FSM_STATUS {
        &self.fsm_status
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

/// FIFO (rw) register accessor: FIFO data register
pub type FIFO = crate::Reg<fifo::FIFO_SPEC>;
pub mod fifo;

/// INT_RAW (rw) register accessor: Raw interrupt status
// 211
pub type INT_RAW = crate::Reg<int_raw::INT_RAW_SPEC>;
pub mod int_raw;

/// INT_ST (r) register accessor: Masked interrupt status
// 215
pub type INT_ST = crate::Reg<int_st::INT_ST_SPEC>;
pub mod int_st;

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

/// STATUS (r) register accessor: UART status register
// 235
pub type STATUS = crate::Reg<status::STATUS_SPEC>;
pub mod status;

/// CONF0 (rw) register accessor: a
// 239
pub type CONF0 = crate::Reg<conf0::CONF0_SPEC>;
pub mod conf0;

/// CONF1 (rw) register accessor: Configuration register 1
// 243
pub type CONF1 = crate::Reg<conf1::CONF1_SPEC>;
pub mod conf1;

/// IDLE_CONF (rw) register accessor: Frame-end idle configuration
// 279
pub type IDLE_CONF = crate::Reg<idle_conf::IDLE_CONF_SPEC>;
pub mod idle_conf;

/// AT_CMD_PRECNT (rw) register accessor: Pre-sequence timing configuration
// 287
pub type AT_CMD_PRECNT = crate::Reg<at_cmd_precnt::AT_CMD_PRECNT_SPEC>;
pub mod at_cmd_precnt;

/// AT_CMD_POSTCNT (rw) register accessor: Post-sequence timing configuration
// 291
pub type AT_CMD_POSTCNT = crate::Reg<at_cmd_postcnt::AT_CMD_POSTCNT_SPEC>;
pub mod at_cmd_postcnt;

/// AT_CMD_GAPTOUT (rw) register accessor: Timeout configuration
// 295
pub type AT_CMD_GAPTOUT = crate::Reg<at_cmd_gaptout::AT_CMD_GAPTOUT_SPEC>;
pub mod at_cmd_gaptout;

/// AT_CMD_CHAR (rw) register accessor: AT escape sequence detection configuration
// 299
pub type AT_CMD_CHAR = crate::Reg<at_cmd_char::AT_CMD_CHAR_SPEC>;
pub mod at_cmd_char;

/// MEM_CONF (rw) register accessor: UART threshold and allocation configuration
// 303
pub type MEM_CONF = crate::Reg<mem_conf::MEM_CONF_SPEC>;
pub mod mem_conf;

/// FSM_STATUS (r) register accessor: UART transmit and receive status.
// 315
pub type FSM_STATUS = crate::Reg<fsm_status::FSM_STATUS_SPEC>;
pub mod fsm_status;

/// CLK_CONF (rw) register accessor: UART core clock configuration
// 327
pub type CLK_CONF = crate::Reg<clk_conf::CLK_CONF_SPEC>;
pub mod clk_conf;

/// ID (rw) register accessor: UART ID register
// 335
pub type ID = crate::Reg<id::ID_SPEC>;
pub mod id;
