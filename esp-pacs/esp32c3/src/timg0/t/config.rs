/// Register `CONFIG` reader
pub type R = crate::R<CONFIG_SPEC>;

/// Register `CONFIG` writer
pub type W = crate::W<CONFIG_SPEC>;

/// Field `USE_XTAL` writer - reg_t0_use_xtal.
pub type USE_XTAL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `ALARM_EN` writer - reg_t0_alarm_en.
pub type ALARM_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DIVIDER` reader - reg_t0_divider.
pub type DIVIDER_R = crate::FieldReader<u16>;

/// Field `AUTORELOAD` reader - reg_t0_autoreload.
pub type AUTORELOAD_R = crate::BitReader;
/// Field `AUTORELOAD` writer - reg_t0_autoreload.
pub type AUTORELOAD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `INCREASE` writer - reg_t0_increase.
pub type INCREASE_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `EN` reader - reg_t0_en.
pub type EN_R = crate::BitReader;
/// Field `EN` writer - reg_t0_en.
pub type EN_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bits 13:28 - reg_t0_divider.
    #[inline(always)]
    pub fn divider(&self) -> DIVIDER_R {
        DIVIDER_R::new(((self.bits >> 13) & 0xffff) as u16)
    }

    /// Bit 29 - reg_t0_autoreload.
    #[inline(always)]
    pub fn autoreload(&self) -> AUTORELOAD_R {
        AUTORELOAD_R::new(((self.bits >> 29) & 1) != 0)
    }

    /// Bit 31 - reg_t0_en.
    #[inline(always)]
    pub fn en(&self) -> EN_R {
        EN_R::new(((self.bits >> 31) & 1) != 0)
    }
}

impl W {
    /// Bit 9 - reg_t0_use_xtal.
    #[inline(always)]
    pub fn use_xtal(&mut self) -> USE_XTAL_W<CONFIG_SPEC> {
        USE_XTAL_W::new(self, 9)
    }

    /// Bit 10 - reg_t0_alarm_en.
    #[inline(always)]
    pub fn alarm_en(&mut self) -> ALARM_EN_W<CONFIG_SPEC> {
        ALARM_EN_W::new(self, 10)
    }

    /// Bit 29 - reg_t0_autoreload.
    #[inline(always)]
    pub fn autoreload(&mut self) -> AUTORELOAD_W<CONFIG_SPEC> {
        AUTORELOAD_W::new(self, 29)
    }

    /// Bit 30 - reg_t0_increase.
    #[inline(always)]
    pub fn increase(&mut self) -> INCREASE_W<CONFIG_SPEC> {
        INCREASE_W::new(self, 30)
    }

    /// Bit 31 - reg_t0_en.
    #[inline(always)]
    pub fn en(&mut self) -> EN_W<CONFIG_SPEC> {
        EN_W::new(self, 31)
    }
}

pub struct CONFIG_SPEC;
impl crate::RegisterSpec for CONFIG_SPEC {
    type Ux = u32;
}

impl crate::Readable for CONFIG_SPEC {}

impl crate::Writable for CONFIG_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
