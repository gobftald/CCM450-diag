/// Register `OUT_W1TC` writer
pub type W = crate::W<OUT_W1TC_SPEC>;

/// Field `OUT_W1TC` writer - GPIO output clear register for GPIO0-25
pub type OUT_W1TC_W<'a, REG> = crate::FieldWriter<'a, REG, 26, u32>;

impl W {
    /// Bits 0:25 - GPIO output clear register for GPIO0-25
    #[inline(always)]
    pub fn out_w1tc(&mut self) -> OUT_W1TC_W<OUT_W1TC_SPEC> {
        OUT_W1TC_W::new(self, 0)
    }
}

pub struct OUT_W1TC_SPEC;
impl crate::RegisterSpec for OUT_W1TC_SPEC {
    type Ux = u32;
}

impl crate::Writable for OUT_W1TC_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for OUT_W1TC_SPEC {
    const RESET_VALUE: u32 = 0;
}
