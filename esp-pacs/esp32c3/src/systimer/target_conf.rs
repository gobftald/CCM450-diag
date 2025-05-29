/// Register `TARGET%s_CONF` reader
pub type R = crate::R<TARGET_CONF_SPEC>;

/// Register `TARGET%s_CONF` writer
pub type W = crate::W<TARGET_CONF_SPEC>;

/// Field `PERIOD` writer - target0 period
pub type PERIOD_W<'a, REG> = crate::FieldWriter<'a, REG, 26, u32>;

/// Field `PERIOD_MODE` reader - Set target0 to period mode
pub type PERIOD_MODE_R = crate::BitReader;
/// Field `PERIOD_MODE` writer - Set target0 to period mode
pub type PERIOD_MODE_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bit 30 - Set target0 to period mode
    #[inline(always)]
    pub fn period_mode(&self) -> PERIOD_MODE_R {
        PERIOD_MODE_R::new(((self.bits >> 30) & 1) != 0)
    }
}

impl W {
    /// Bits 0:25 - target0 period
    #[inline(always)]
    pub fn period(&mut self) -> PERIOD_W<TARGET_CONF_SPEC> {
        PERIOD_W::new(self, 0)
    }

    /// Bit 30 - Set target0 to period mode
    #[inline(always)]
    pub fn period_mode(&mut self) -> PERIOD_MODE_W<TARGET_CONF_SPEC> {
        PERIOD_MODE_W::new(self, 30)
    }
}

pub struct TARGET_CONF_SPEC;
impl crate::RegisterSpec for TARGET_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for TARGET_CONF_SPEC {}

impl crate::Writable for TARGET_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
