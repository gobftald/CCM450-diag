/// Register `ENABLE_W1TC` writer
pub type W = crate::W<ENABLE_W1TC_SPEC>;

/// Field `ENABLE_W1TC` writer - GPIO output enable clear register for GPIO0-25
pub type ENABLE_W1TC_W<'a, REG> = crate::FieldWriter<'a, REG, 26, u32>;

impl W {
    /// Bits 0:25 - GPIO output enable clear register for GPIO0-25
    #[inline(always)]
    pub fn enable_w1tc(&mut self) -> ENABLE_W1TC_W<ENABLE_W1TC_SPEC> {
        ENABLE_W1TC_W::new(self, 0)
    }
}

pub struct ENABLE_W1TC_SPEC;
impl crate::RegisterSpec for ENABLE_W1TC_SPEC {
    type Ux = u32;
}

impl crate::Writable for ENABLE_W1TC_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for ENABLE_W1TC_SPEC {
    const RESET_VALUE: u32 = 0;
}
