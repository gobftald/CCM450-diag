/// Register `LO` reader
pub type R = crate::R<LO_SPEC>;

/// Field `VALUE_LO` reader - timer read value low 32bit
pub type VALUE_LO_R = crate::FieldReader<u32>;

impl R {
    /// Bits 0:31 - timer read value low 32bit
    #[inline(always)]
    pub fn value_lo(&self) -> VALUE_LO_R {
        VALUE_LO_R::new(self.bits)
    }
}

pub struct LO_SPEC;
impl crate::RegisterSpec for LO_SPEC {
    type Ux = u32;
}

impl crate::Readable for LO_SPEC {}
