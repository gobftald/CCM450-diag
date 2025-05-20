/// Register `STORE5` writer
pub type W = crate::W<STORE5_SPEC>;

/// Field `SCRATCH5` writer - reserved register
pub type SCRATCH5_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - reserved register
    #[inline(always)]
    pub fn scratch5(&mut self) -> SCRATCH5_W<STORE5_SPEC> {
        SCRATCH5_W::new(self, 0)
    }
}

pub struct STORE5_SPEC;
impl crate::RegisterSpec for STORE5_SPEC {
    type Ux = u32;
}

impl crate::Readable for STORE5_SPEC {}

impl crate::Writable for STORE5_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
