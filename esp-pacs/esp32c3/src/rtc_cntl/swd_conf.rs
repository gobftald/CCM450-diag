/// Register `SWD_CONF` writer
pub type W = crate::W<SWD_CONF_SPEC>;

/// Field `SWD_AUTO_FEED_EN` writer - automatically feed swd when int comes
pub type SWD_AUTO_FEED_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 31 - automatically feed swd when int comes
    #[inline(always)]
    pub fn swd_auto_feed_en(&mut self) -> SWD_AUTO_FEED_EN_W<SWD_CONF_SPEC> {
        SWD_AUTO_FEED_EN_W::new(self, 31)
    }
}

pub struct SWD_CONF_SPEC;
impl crate::RegisterSpec for SWD_CONF_SPEC {
    type Ux = u32;
}

impl crate::Writable for SWD_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

/// `reset()` method sets SWD_CONF to value 0x04b0_0000"]
impl crate::Resettable for SWD_CONF_SPEC {
    const RESET_VALUE: u32 = 0x04b0_0000;
}
