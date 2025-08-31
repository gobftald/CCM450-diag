#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x08],
    out_w1ts: OUT_W1TS,
    out_w1tc: OUT_W1TC,
    _reserved1: [u8; 0x14],
    enable_w1ts: ENABLE_W1TS,
    enable_w1tc: ENABLE_W1TC,
    _reserved2: [u8; 0x48],
    pin: [PIN; 26],
    _reserved3: [u8; 0x78],
    func_in_sel_cfg: [FUNC_IN_SEL_CFG; 128],
    _reserved4: [u8; 0x200],
    func_out_sel_cfg: [FUNC_OUT_SEL_CFG; 26],
}

impl RegisterBlock {
    /// 0x08 - GPIO output set register
    #[inline(always)]
    pub const fn out_w1ts(&self) -> &OUT_W1TS {
        &self.out_w1ts
    }

    /// 0x0c - GPIO output clear register
    #[inline(always)]
    pub const fn out_w1tc(&self) -> &OUT_W1TC {
        &self.out_w1tc
    }

    /// 0x24 - GPIO output enable set register
    #[inline(always)]
    pub const fn enable_w1ts(&self) -> &ENABLE_W1TS {
        &self.enable_w1ts
    }

    // 0x28 - GPIO output enable clear register
    #[inline(always)]
    pub const fn enable_w1tc(&self) -> &ENABLE_W1TC {
        &self.enable_w1tc
    }

    /// 0x74..0xdc - GPIO pin configuration register
    #[inline(always)]
    pub const fn pin(&self, n: usize) -> &PIN {
        &self.pin[n]
    }

    /// 0x154..0x354 - GPIO input function configuration register
    #[inline(always)]
    pub const fn func_in_sel_cfg(&self, n: usize) -> &FUNC_IN_SEL_CFG {
        &self.func_in_sel_cfg[n]
    }

    /// 0x554..0x5bc - GPIO output function select register
    #[inline(always)]
    pub const fn func_out_sel_cfg(&self, n: usize) -> &FUNC_OUT_SEL_CFG {
        &self.func_out_sel_cfg[n]
    }
}

/// OUT_W1TS (w) register accessor: GPIO output set register
// 947
pub type OUT_W1TS = crate::Reg<out_w1ts::OUT_W1TS_SPEC>;
pub mod out_w1ts;

/// OUT_W1TC (w) register accessor: GPIO output clear register
// 951
pub type OUT_W1TC = crate::Reg<out_w1tc::OUT_W1TC_SPEC>;
pub mod out_w1tc;

/// ENABLE_W1TS (w) register accessor: GPIO output enable set register
// 963
pub type ENABLE_W1TS = crate::Reg<enable_w1ts::ENABLE_W1TS_SPEC>;
pub mod enable_w1ts;

/// ENABLE_W1TC (w) register accessor: GPIO output enable clear register
// 967
pub type ENABLE_W1TC = crate::Reg<enable_w1tc::ENABLE_W1TC_SPEC>;
#[doc = "GPIO output enable clear register"]
pub mod enable_w1tc;

/// PIN (rw) register accessor: GPIO pin configuration register
pub type PIN = crate::Reg<pin::PIN_SPEC>;
pub mod pin;

/// FUNC_IN_SEL_CFG (rw) register accessor: GPIO input function configuration register
// 1011
pub type FUNC_IN_SEL_CFG = crate::Reg<func_in_sel_cfg::FUNC_IN_SEL_CFG_SPEC>;
pub mod func_in_sel_cfg;

/// FUNC_OUT_SEL_CFG (rw) register accessor: GPIO output function select register
// 1015
pub type FUNC_OUT_SEL_CFG = crate::Reg<func_out_sel_cfg::FUNC_OUT_SEL_CFG_SPEC>;
pub mod func_out_sel_cfg;
