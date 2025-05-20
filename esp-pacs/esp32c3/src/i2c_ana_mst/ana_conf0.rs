/// Register `ANA_CONF0` writer
pub type W = crate::W<ANA_CONF0_SPEC>;

/// Field `BBPLL_STOP_FORCE_HIGH` writer - ?
pub type BBPLL_STOP_FORCE_HIGH_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BBPLL_STOP_FORCE_LOW` writer - ?
pub type BBPLL_STOP_FORCE_LOW_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 2 - ?
    #[inline(always)]
    pub fn bbpll_stop_force_high(&mut self) -> BBPLL_STOP_FORCE_HIGH_W<ANA_CONF0_SPEC> {
        BBPLL_STOP_FORCE_HIGH_W::new(self, 2)
    }

    #[doc = "Bit 3 - ?"]
    #[inline(always)]
    pub fn bbpll_stop_force_low(&mut self) -> BBPLL_STOP_FORCE_LOW_W<ANA_CONF0_SPEC> {
        BBPLL_STOP_FORCE_LOW_W::new(self, 3)
    }
}
pub struct ANA_CONF0_SPEC;

impl crate::RegisterSpec for ANA_CONF0_SPEC {
    type Ux = u32;
}

impl crate::Readable for ANA_CONF0_SPEC {}

impl crate::Writable for ANA_CONF0_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
