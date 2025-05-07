/// Register `INTR_STATUS_REG_1` reader
pub type R = crate::R<INTR_STATUS_REG_1_SPEC>;

/// Field `INTR_STATUS_1` reader - reg_core0_intr_status_1
pub type INTR_STATUS_1_R = crate::FieldReader<u32>;

impl R {
    /// Bits 0:31 - reg_core0_intr_status_1
    #[inline(always)]
    pub fn intr_status_1(&self) -> INTR_STATUS_1_R {
        INTR_STATUS_1_R::new(self.bits)
    }
}

pub struct INTR_STATUS_REG_1_SPEC;
impl crate::RegisterSpec for INTR_STATUS_REG_1_SPEC {
    type Ux = u32;
}

impl crate::Readable for INTR_STATUS_REG_1_SPEC {}
