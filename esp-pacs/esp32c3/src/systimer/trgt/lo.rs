/// Register `LO` writer
pub type W = crate::W<LO_SPEC>;

/// Field `LO` writer - timer taget0 low 32 bit
pub type LO_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32, crate::Safe>;

impl W {
    /// Bits 0:31 - timer taget0 low 32 bit
    #[inline(always)]
    pub fn lo(&mut self) -> LO_W<LO_SPEC> {
        LO_W::new(self, 0)
    }
}

pub struct LO_SPEC;
impl crate::RegisterSpec for LO_SPEC {
    type Ux = u32;
}

impl crate::Writable for LO_SPEC {
    type Safety = crate::Safe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for LO_SPEC {
    const RESET_VALUE: u32 = 0;
}
