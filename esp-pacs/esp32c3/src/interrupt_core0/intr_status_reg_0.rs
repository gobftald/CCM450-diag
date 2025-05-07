/// Register `INTR_STATUS_REG_0` reader
pub type R = crate::R<INTR_STATUS_REG_0_SPEC>;

/// Field `INTR_STATUS_0` reader - reg_core0_intr_status_0
pub type INTR_STATUS_0_R = crate::FieldReader<u32>;

impl R {
    /// Bits 0:31 - reg_core0_intr_status_0
    #[inline(always)]
    pub fn intr_status_0(&self) -> INTR_STATUS_0_R {
        INTR_STATUS_0_R::new(self.bits)
    }
}

pub struct INTR_STATUS_REG_0_SPEC;
impl crate::RegisterSpec for INTR_STATUS_REG_0_SPEC {
    type Ux = u32;
}
impl crate::Readable for INTR_STATUS_REG_0_SPEC {}
