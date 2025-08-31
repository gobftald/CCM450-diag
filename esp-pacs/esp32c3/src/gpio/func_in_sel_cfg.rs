/// Register `FUNC%s_IN_SEL_CFG` writer
pub type W = crate::W<FUNC_IN_SEL_CFG_SPEC>;

/// Field `IN_SEL` writer - set this value: s=0-53: connect GPIO[s] to this port.
/// s=0x38: set this port always high level. s=0x3C: set this port always low level.
pub type IN_SEL_W<'a, REG> = crate::FieldWriter<'a, REG, 5>;

/// Field `IN_INV_SEL` writer - set this bit to invert input signal. 1:invert. 0:not invert.
pub type IN_INV_SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `SEL` writer - set this bit to bypass GPIO. 1:do not bypass GPIO. 0:bypass GPIO.
pub type SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bits 0:4 - set this value: s=0-53: connect GPIO[s] to this port.
    /// s=0x38: set this port always high level. s=0x3C: set this port always low level.
    #[inline(always)]
    pub fn in_sel(&mut self) -> IN_SEL_W<FUNC_IN_SEL_CFG_SPEC> {
        IN_SEL_W::new(self, 0)
    }

    /// Bit 5 - set this bit to invert input signal. 1:invert. 0:not invert.
    #[inline(always)]
    pub fn in_inv_sel(&mut self) -> IN_INV_SEL_W<FUNC_IN_SEL_CFG_SPEC> {
        IN_INV_SEL_W::new(self, 5)
    }

    /// Bit 6 - set this bit to bypass GPIO. 1:do not bypass GPIO. 0:bypass GPIO.
    #[inline(always)]
    pub fn sel(&mut self) -> SEL_W<FUNC_IN_SEL_CFG_SPEC> {
        SEL_W::new(self, 6)
    }
}

pub struct FUNC_IN_SEL_CFG_SPEC;
impl crate::RegisterSpec for FUNC_IN_SEL_CFG_SPEC {
    type Ux = u32;
}

impl crate::Writable for FUNC_IN_SEL_CFG_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for FUNC_IN_SEL_CFG_SPEC {
    const RESET_VALUE: u32 = 0;
}
