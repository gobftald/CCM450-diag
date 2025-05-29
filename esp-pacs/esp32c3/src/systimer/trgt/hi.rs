/// Register `HI` writer
pub type W = crate::W<HI_SPEC>;
/// Field `HI` writer - timer taget0 high 32 bit
pub type HI_W<'a, REG> = crate::FieldWriter<'a, REG, 20, u32, crate::Safe>;

impl W {
    /// Bits 0:19 - timer taget0 high 32 bit
    #[inline(always)]
    pub fn hi(&mut self) -> HI_W<HI_SPEC> {
        HI_W::new(self, 0)
    }
}

pub struct HI_SPEC;
impl crate::RegisterSpec for HI_SPEC {
    type Ux = u32;
}

impl crate::Writable for HI_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for HI_SPEC {
    const RESET_VALUE: u32 = 0;
}
