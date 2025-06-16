/// Register `LOADHI` writer
pub type W = crate::W<LOADHI_SPEC>;

/// Field `LOAD_HI` writer - reg_t0_load_hi.
pub type LOAD_HI_W<'a, REG> = crate::FieldWriter<'a, REG, 22, u32>;

impl W {
    /// Bits 0:21 - reg_t0_load_hi.
    #[inline(always)]
    pub fn load_hi(&mut self) -> LOAD_HI_W<LOADHI_SPEC> {
        LOAD_HI_W::new(self, 0)
    }
}

pub struct LOADHI_SPEC;
impl crate::RegisterSpec for LOADHI_SPEC {
    type Ux = u32;
}

impl crate::Writable for LOADHI_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for LOADHI_SPEC {
    const RESET_VALUE: u32 = 0;
}
