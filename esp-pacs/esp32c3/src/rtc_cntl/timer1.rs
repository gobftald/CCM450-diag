/// Register `TIMER1` writer
pub type W = crate::W<TIMER1_SPEC>;

/// Field `CK8M_WAIT` writer - CK8M wait cycles in slow_clk_rtc
pub type CK8M_WAIT_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

/// Field `PLL_BUF_WAIT` writer - PLL wait cycles in slow_clk_rtc
pub type PLL_BUF_WAIT_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

impl W {
    /// Bits 6:13 - CK8M wait cycles in slow_clk_rtc
    #[inline(always)]
    pub fn ck8m_wait(&mut self) -> CK8M_WAIT_W<TIMER1_SPEC> {
        CK8M_WAIT_W::new(self, 6)
    }

    /// Bits 24:31 - PLL wait cycles in slow_clk_rtc
    #[inline(always)]
    pub fn pll_buf_wait(&mut self) -> PLL_BUF_WAIT_W<TIMER1_SPEC> {
        PLL_BUF_WAIT_W::new(self, 24)
    }
}

pub struct TIMER1_SPEC;
impl crate::RegisterSpec for TIMER1_SPEC {
    type Ux = u32;
}

impl crate::Readable for TIMER1_SPEC {}

impl crate::Writable for TIMER1_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
