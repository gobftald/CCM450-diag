/// "Register `COMP%s_LOAD` writer
pub type W = crate::W<COMP_LOAD_SPEC>;

/// Field `LOAD` writer - timer comp0 load value
pub type LOAD_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 0 - timer comp0 load value
    #[inline(always)]
    pub fn load(&mut self) -> LOAD_W<COMP_LOAD_SPEC> {
        LOAD_W::new(self, 0)
    }
}

pub struct COMP_LOAD_SPEC;
impl crate::RegisterSpec for COMP_LOAD_SPEC {
    type Ux = u32;
}

impl crate::Writable for COMP_LOAD_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for COMP_LOAD_SPEC {
    const RESET_VALUE: u32 = 0;
}
