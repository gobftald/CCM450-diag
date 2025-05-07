/// Register `HI` reader
pub type R = crate::R<HI_SPEC>;

/// Field `VALUE_HI` reader - timer read value high 32bit
pub type VALUE_HI_R = crate::FieldReader<u32>;

impl R {
    /// Bits 0:19 - timer read value high 32bit
    #[inline(always)]
    pub fn value_hi(&self) -> VALUE_HI_R {
        VALUE_HI_R::new(self.bits & 0x000f_ffff)
    }
}

pub struct HI_SPEC;
impl crate::RegisterSpec for HI_SPEC {
    type Ux = u32;
}

impl crate::Readable for HI_SPEC {}
