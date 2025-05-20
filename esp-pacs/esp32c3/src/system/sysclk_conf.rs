/// Register `SYSCLK_CONF` writer
pub type W = crate::W<SYSCLK_CONF_SPEC>;

/// Field `PRE_DIV_CNT` writer - reg_pre_div_cnt
pub type PRE_DIV_CNT_W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;

/// Field `SOC_CLK_SEL` writer - reg_soc_clk_sel
pub type SOC_CLK_SEL_W<'a, REG> = crate::FieldWriter<'a, REG, 2>;

impl W {
    /// Bits 0:9 - reg_pre_div_cnt
    #[inline(always)]
    pub fn pre_div_cnt(&mut self) -> PRE_DIV_CNT_W<SYSCLK_CONF_SPEC> {
        PRE_DIV_CNT_W::new(self, 0)
    }

    /// Bits 10:11 - reg_soc_clk_sel
    #[inline(always)]
    pub fn soc_clk_sel(&mut self) -> SOC_CLK_SEL_W<SYSCLK_CONF_SPEC> {
        SOC_CLK_SEL_W::new(self, 10)
    }
}

pub struct SYSCLK_CONF_SPEC;
impl crate::RegisterSpec for SYSCLK_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for SYSCLK_CONF_SPEC {}

impl crate::Writable for SYSCLK_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
