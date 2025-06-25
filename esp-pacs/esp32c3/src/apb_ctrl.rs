#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x14],
    wifi_clk_en: WIFI_CLK_EN,
    wifi_rst_en: WIFI_RST_EN,
}

impl RegisterBlock {
    /// 0x14 - APB_CTRL_WIFI_CLK_EN_REG
    #[inline(always)]
    pub const fn wifi_clk_en(&self) -> &WIFI_CLK_EN {
        &self.wifi_clk_en
    }

    /// 0x18 - APB_CTRL_WIFI_RST_EN_REG
    #[inline(always)]
    pub const fn wifi_rst_en(&self) -> &WIFI_RST_EN {
        &self.wifi_rst_en
    }
}

/// WIFI_CLK_EN (rw) register accessor: APB_CTRL_WIFI_CLK_EN_REG
// 273
pub type WIFI_CLK_EN = crate::Reg<wifi_clk_en::WIFI_CLK_EN_SPEC>;
pub mod wifi_clk_en;

/// WIFI_RST_EN (rw) register accessor: APB_CTRL_WIFI_RST_EN_REG
// 277
pub type WIFI_RST_EN = crate::Reg<wifi_rst_en::WIFI_RST_EN_SPEC>;
pub mod wifi_rst_en;
