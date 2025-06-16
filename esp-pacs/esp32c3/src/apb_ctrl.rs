#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
#[doc = "Register block"]
pub struct RegisterBlock {
    _reserved0: [u8; 0x18],
    wifi_rst_en: WIFI_RST_EN,
}

impl RegisterBlock {
    /// 0x18 - APB_CTRL_WIFI_RST_EN_REG
    #[inline(always)]
    pub const fn wifi_rst_en(&self) -> &WIFI_RST_EN {
        &self.wifi_rst_en
    }
}

/// WIFI_RST_EN (rw) register accessor: APB_CTRL_WIFI_RST_EN_REG
// 277
pub type WIFI_RST_EN = crate::Reg<wifi_rst_en::WIFI_RST_EN_SPEC>;
pub mod wifi_rst_en;
