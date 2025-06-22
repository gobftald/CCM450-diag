#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x08],
    cpu_per_conf: CPU_PER_CONF,
    mem_pd_mask: MEM_PD_MASK,
    perip_clk_en0: PERIP_CLK_EN0,
    _reserved1: [u8; 0x04],
    perip_rst_en0: PERIP_RST_EN0,
    _reserved2: [u8; 0x14],
    cpu_intr_from_cpu_2: CPU_INTR_FROM_CPU_2,
    _reserved3: [u8; 0x24],
    sysclk_conf: SYSCLK_CONF,
}

impl RegisterBlock {
    /// 0x08 - cpu clock config register
    #[inline(always)]
    pub const fn cpu_per_conf(&self) -> &CPU_PER_CONF {
        &self.cpu_per_conf
    }

    /// 0x0c - memory power down mask register
    #[inline(always)]
    pub const fn mem_pd_mask(&self) -> &MEM_PD_MASK {
        &self.mem_pd_mask
    }

    /// 0x10 - peripheral clock gating register
    #[inline(always)]
    pub const fn perip_clk_en0(&self) -> &PERIP_CLK_EN0 {
        &self.perip_clk_en0
    }

    /// 0x18 - reserved
    #[inline(always)]
    pub const fn perip_rst_en0(&self) -> &PERIP_RST_EN0 {
        &self.perip_rst_en0
    }

    /// 0x30 - interrupt generate register
    #[inline(always)]
    pub const fn cpu_intr_from_cpu_2(&self) -> &CPU_INTR_FROM_CPU_2 {
        &self.cpu_intr_from_cpu_2
    }

    /// 0x58 - system clock config register
    #[inline(always)]
    pub const fn sysclk_conf(&self) -> &SYSCLK_CONF {
        &self.sysclk_conf
    }
}

/// CPU_PER_CONF (rw) register accessor: cpu clock config register
// 260
pub type CPU_PER_CONF = crate::Reg<cpu_per_conf::CPU_PER_CONF_SPEC>;
pub mod cpu_per_conf;

/// MEM_PD_MASK (rw) register accessor: memory power down mask register
// 264
pub type MEM_PD_MASK = crate::Reg<mem_pd_mask::MEM_PD_MASK_SPEC>;
pub mod mem_pd_mask;

/// PERIP_CLK_EN0 (rw) register accessor: peripheral clock gating register
// 268
pub type PERIP_CLK_EN0 = crate::Reg<perip_clk_en0::PERIP_CLK_EN0_SPEC>;
pub mod perip_clk_en0;

/// PERIP_RST_EN0 (rw) register accessor: reserved
// 276
pub type PERIP_RST_EN0 = crate::Reg<perip_rst_en0::PERIP_RST_EN0_SPEC>;
pub mod perip_rst_en0;

/// CPU_INTR_FROM_CPU_2 (rw) register accessor: interrupt generate register
// 300
pub type CPU_INTR_FROM_CPU_2 = crate::Reg<cpu_intr_from_cpu_2::CPU_INTR_FROM_CPU_2_SPEC>;
pub mod cpu_intr_from_cpu_2;

/// SYSCLK_CONF (rw) register accessor: system clock config register
// 342
pub type SYSCLK_CONF = crate::Reg<sysclk_conf::SYSCLK_CONF_SPEC>;
pub mod sysclk_conf;
