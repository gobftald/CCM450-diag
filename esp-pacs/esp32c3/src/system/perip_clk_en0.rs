/// Register `PERIP_CLK_EN0` writer
pub type W = crate::W<PERIP_CLK_EN0_SPEC>;

/// Field `SYSTIMER_CLK_EN` writer - reg_systimer_clk_en
pub type SYSTIMER_CLK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
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
