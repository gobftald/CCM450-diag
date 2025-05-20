/// Register `TIMER5` writer
pub type W = crate::W<TIMER5_SPEC>;

/// Field `MIN_SLP_VAL` writer - minimal sleep cycles in slow_clk_rtc
pub type MIN_SLP_VAL_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

impl W {
    /// Bits 8:15 - minimal sleep cycles in slow_clk_rtc
    #[inline(always)]
    pub fn min_slp_val(&mut self) -> MIN_SLP_VAL_W<TIMER5_SPEC> {
        MIN_SLP_VAL_W::new(self, 8)
    }
}

pub struct TIMER5_SPEC;
impl crate::RegisterSpec for TIMER5_SPEC {
    type Ux = u32;
}

impl crate::Readable for TIMER5_SPEC {}

impl crate::Writable for TIMER5_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
