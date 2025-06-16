/// Register `LOAD` writer
pub type W = crate::W<LOAD_SPEC>;

/// Field `LOAD` writer - t0_load
pub type LOAD_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - t0_load
    #[inline(always)]
    pub fn load(&mut self) -> LOAD_W<LOAD_SPEC> {
        LOAD_W::new(self, 0)
    }
}

pub struct LOAD_SPEC;
impl crate::RegisterSpec for LOAD_SPEC {
    type Ux = u32;
}

impl crate::Writable for LOAD_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for LOAD_SPEC {
    const RESET_VALUE: u32 = 0;
}
