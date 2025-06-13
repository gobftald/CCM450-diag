/// Register `PERIP_CLK_EN0` writer
pub type W = crate::W<PERIP_CLK_EN0_SPEC>;

/// Field `TIMERS_CLK_EN` writer - reg_timers_clk_en
pub type TIMERS_CLK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TIMERGROUP_CLK_EN` writer - reg_timergroup_clk_en
pub type TIMERGROUP_CLK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `SYSTIMER_CLK_EN` writer - reg_systimer_clk_en
pub type SYSTIMER_CLK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 0 - reg_timers_clk_en
    #[inline(always)]
    pub fn timers_clk_en(&mut self) -> TIMERS_CLK_EN_W<PERIP_CLK_EN0_SPEC> {
        TIMERS_CLK_EN_W::new(self, 0)
    }

    /// Bit 13 - reg_timergroup_clk_en
    #[inline(always)]
    pub fn timergroup_clk_en(&mut self) -> TIMERGROUP_CLK_EN_W<PERIP_CLK_EN0_SPEC> {
        TIMERGROUP_CLK_EN_W::new(self, 13)
    }

    /// Bit 29 - reg_systimer_clk_en
    #[inline(always)]
    pub fn systimer_clk_en(&mut self) -> SYSTIMER_CLK_EN_W<PERIP_CLK_EN0_SPEC> {
        SYSTIMER_CLK_EN_W::new(self, 29)
    }
}

pub struct PERIP_CLK_EN0_SPEC;
impl crate::RegisterSpec for PERIP_CLK_EN0_SPEC {
    type Ux = u32;
}

impl crate::Readable for PERIP_CLK_EN0_SPEC {}

impl crate::Writable for PERIP_CLK_EN0_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
