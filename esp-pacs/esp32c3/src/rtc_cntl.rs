#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
#[doc = "Register block"]
pub struct RegisterBlock {
    options0: OPTIONS0,
    _reserved0: [u8; 0x18],
    timer1: TIMER1,
    _reserved1: [u8; 0x04],
    timer3: TIMER3,
    timer4: TIMER4,
    timer5: TIMER5,
    timer6: TIMER6,
    ana_conf: ANA_CONF,
    _reserved2: [u8; 0x08],
    int_ena: INT_ENA,
    _reserved3: [u8; 0x08],
    int_clr: INT_CLR,
    _reserved4: [u8; 0x20],
    clk_conf: CLK_CONF,
    _reserved5: [u8; 0x0c],
    rtc_cntl: RTC_CNTL,
    _reserved6: [u8; 0x04],
    dig_pwc: DIG_PWC,
    dig_iso: DIG_ISO,
    wdtconfig0: WDTCONFIG0,
    _reserved7: [u8; 0x14],
    wdtwprotect: WDTWPROTECT,
    swd_conf: SWD_CONF,
    swd_wprotect: SWD_WPROTECT,
    _reserved8: [u8; 0x08],
    store5: STORE5,
}

impl RegisterBlock {
    /// 0x00 - rtc configure register
    #[inline(always)]
    pub const fn options0(&self) -> &OPTIONS0 {
        &self.options0
    }

    /// 0x1c - rtc configure register
    #[inline(always)]
    pub const fn timer1(&self) -> &TIMER1 {
        &self.timer1
    }

    /// 0x24 - rtc configure register
    #[inline(always)]
    pub const fn timer3(&self) -> &TIMER3 {
        &self.timer3
    }

    /// 0x28 - rtc configure register
    #[inline(always)]
    pub const fn timer4(&self) -> &TIMER4 {
        &self.timer4
    }

    /// 0x2c - rtc configure register
    #[inline(always)]
    pub const fn timer5(&self) -> &TIMER5 {
        &self.timer5
    }

    /// 0x30 - rtc configure register
    #[inline(always)]
    pub const fn timer6(&self) -> &TIMER6 {
        &self.timer6
    }

    /// 0x34 - rtc configure register
    #[inline(always)]
    pub const fn ana_conf(&self) -> &ANA_CONF {
        &self.ana_conf
    }

    /// 0x40 - rtc configure register
    #[inline(always)]
    pub const fn int_ena(&self) -> &INT_ENA {
        &self.int_ena
    }

    /// 0x4c - rtc configure register
    #[inline(always)]
    pub const fn int_clr(&self) -> &INT_CLR {
        &self.int_clr
    }

    /// 0x70 - rtc configure register
    #[inline(always)]
    pub const fn clk_conf(&self) -> &CLK_CONF {
        &self.clk_conf
    }

    /// 0x80 - rtc configure register
    #[inline(always)]
    pub const fn rtc_cntl(&self) -> &RTC_CNTL {
        &self.rtc_cntl
    }

    /// 0x88 - rtc configure register
    #[inline(always)]
    pub const fn dig_pwc(&self) -> &DIG_PWC {
        &self.dig_pwc
    }

    /// 0x8c - rtc configure register
    #[inline(always)]
    pub const fn dig_iso(&self) -> &DIG_ISO {
        &self.dig_iso
    }

    /// 0x90 - rtc configure register
    #[inline(always)]
    pub const fn wdtconfig0(&self) -> &WDTCONFIG0 {
        &self.wdtconfig0
    }

    /// 0xa8 - rtc configure register
    #[inline(always)]
    pub const fn wdtwprotect(&self) -> &WDTWPROTECT {
        &self.wdtwprotect
    }

    ///"0xac - rtc configure register
    #[inline(always)]
    pub const fn swd_conf(&self) -> &SWD_CONF {
        &self.swd_conf
    }

    /// 0xb0 - rtc configure register
    #[inline(always)]
    pub const fn swd_wprotect(&self) -> &SWD_WPROTECT {
        &self.swd_wprotect
    }

    /// 0xbc - rtc configure register
    #[inline(always)]
    pub const fn store5(&self) -> &STORE5 {
        &self.store5
    }
}

/// OPTIONS0 (rw) register accessor: rtc configure register
// 460
pub type OPTIONS0 = crate::Reg<options0::OPTIONS0_SPEC>;
pub mod options0;

/// TIMER1 (rw) register accessor: rtc configure register
// 488
pub type TIMER1 = crate::Reg<timer1::TIMER1_SPEC>;
pub mod timer1;

/// TIMER3 (rw) register accessor: rtc configure register
// 496
pub type TIMER3 = crate::Reg<timer3::TIMER3_SPEC>;
pub mod timer3;

/// TIMER4 (rw) register accessor: rtc configure register
// 500
pub type TIMER4 = crate::Reg<timer4::TIMER4_SPEC>;
pub mod timer4;

/// TIMER5 (rw) register accessor: rtc configure register
// 504
pub type TIMER5 = crate::Reg<timer5::TIMER5_SPEC>;
pub mod timer5;

/// TIMER6 (rw) register accessor: rtc configure register
// 508
pub type TIMER6 = crate::Reg<timer6::TIMER6_SPEC>;
pub mod timer6;

/// ANA_CONF (rw) register accessor: rtc configure register
// 512
pub type ANA_CONF = crate::Reg<ana_conf::ANA_CONF_SPEC>;
pub mod ana_conf;

/// INT_ENA (rw) register accessor: rtc configure register
// 524
pub type INT_ENA = crate::Reg<int_ena::INT_ENA_SPEC>;
pub mod int_ena;

/// INT_CLR (w) register accessor: rtc configure register
// 536
pub type INT_CLR = crate::Reg<int_clr::INT_CLR_SPEC>;
pub mod int_clr;

/// CLK_CONF (rw) register accessor: rtc configure register
// 572
pub type CLK_CONF = crate::Reg<clk_conf::CLK_CONF_SPEC>;
pub mod clk_conf;

/// RTC_CNTL (rw) register accessor: rtc configure register
// 588
pub type RTC_CNTL = crate::Reg<rtc_cntl::RTC_CNTL_SPEC>;
pub mod rtc_cntl;

/// DIG_PWC (rw) register accessor: rtc configure register
// 596
pub type DIG_PWC = crate::Reg<dig_pwc::DIG_PWC_SPEC>;
pub mod dig_pwc;

/// DIG_ISO (rw) register accessor: rtc configure register
// 600
pub type DIG_ISO = crate::Reg<dig_iso::DIG_ISO_SPEC>;
#[doc = "rtc configure register"]
pub mod dig_iso;

/// WDTCONFIG0 (rw) register accessor: rtc configure register
// 604
pub type WDTCONFIG0 = crate::Reg<wdtconfig0::WDTCONFIG0_SPEC>;
pub mod wdtconfig0;

/// WDTWPROTECT (rw) register accessor: rtc configure register
// 628
pub type WDTWPROTECT = crate::Reg<wdtwprotect::WDTWPROTECT_SPEC>;
pub mod wdtwprotect;

/// SWD_CONF (rw) register accessor: rtc configure register
// 632
pub type SWD_CONF = crate::Reg<swd_conf::SWD_CONF_SPEC>;
pub mod swd_conf;

/// SWD_WPROTECT (rw) register accessor: rtc configure register
// 636
pub type SWD_WPROTECT = crate::Reg<swd_wprotect::SWD_WPROTECT_SPEC>;
pub mod swd_wprotect;

/// STORE5 (rw) register accessor: rtc configure register
// 648
pub type STORE5 = crate::Reg<store5::STORE5_SPEC>;
pub mod store5;
