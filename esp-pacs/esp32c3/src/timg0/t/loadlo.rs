/// Register `LOADLO` writer
pub type W = crate::W<LOADLO_SPEC>;

/// Field `LOAD_LO` writer - reg_t0_load_lo.
pub type LOAD_LO_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - reg_t0_load_lo.
    #[inline(always)]
    pub fn load_lo(&mut self) -> LOAD_LO_W<LOADLO_SPEC> {
        LOAD_LO_W::new(self, 0)
    }
}

pub struct LOADLO_SPEC;
impl crate::RegisterSpec for LOADLO_SPEC {
    type Ux = u32;
}

impl crate::Writable for LOADLO_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for LOADLO_SPEC {
    const RESET_VALUE: u32 = 0;
}
