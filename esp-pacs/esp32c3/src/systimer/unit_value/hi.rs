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

/// SYSTIMER_UNIT0_VALUE_HI.
///
/// You can [`read`](crate::Reg::read) this register and get [`hi::R`](R).
/// See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct HI_SPEC;
impl crate::RegisterSpec for HI_SPEC {
    type Ux = u32;
}

/// `read()` method returns [`hi::R`](R) reader structure
impl crate::Readable for HI_SPEC {}
