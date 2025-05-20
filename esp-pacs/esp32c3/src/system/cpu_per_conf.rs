/// Register `CPU_PER_CONF` writer
pub type W = crate::W<CPU_PER_CONF_SPEC>;

/// Field `CPUPERIOD_SEL` writer - reg_cpuperiod_sel
pub type CPUPERIOD_SEL_W<'a, REG> = crate::FieldWriter<'a, REG, 2>;

/// Field `PLL_FREQ_SEL` writer - reg_pll_freq_sel
pub type PLL_FREQ_SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `CPU_WAIT_MODE_FORCE_ON` writer - reg_cpu_wait_mode_force_on
pub type CPU_WAIT_MODE_FORCE_ON_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bits 0:1 - reg_cpuperiod_sel
    #[inline(always)]
    pub fn cpuperiod_sel(&mut self) -> CPUPERIOD_SEL_W<CPU_PER_CONF_SPEC> {
        CPUPERIOD_SEL_W::new(self, 0)
    }

    /// Bit 2 - reg_pll_freq_sel
    #[inline(always)]
    pub fn pll_freq_sel(&mut self) -> PLL_FREQ_SEL_W<CPU_PER_CONF_SPEC> {
        PLL_FREQ_SEL_W::new(self, 2)
    }

    /// Bit 3 - reg_cpu_wait_mode_force_on
    #[inline(always)]
    pub fn cpu_wait_mode_force_on(&mut self) -> CPU_WAIT_MODE_FORCE_ON_W<CPU_PER_CONF_SPEC> {
        CPU_WAIT_MODE_FORCE_ON_W::new(self, 3)
    }
}

pub struct CPU_PER_CONF_SPEC;
impl crate::RegisterSpec for CPU_PER_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for CPU_PER_CONF_SPEC {}

impl crate::Writable for CPU_PER_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
