/// Register `CONFIG` writer
pub type W = crate::W<CONFIG_SPEC>;

/// Field `USE_XTAL` writer - reg_t0_use_xtal.
pub type USE_XTAL_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 9 - reg_t0_use_xtal.
    #[inline(always)]
    pub fn use_xtal(&mut self) -> USE_XTAL_W<CONFIG_SPEC> {
        USE_XTAL_W::new(self, 9)
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
