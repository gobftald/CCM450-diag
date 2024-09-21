/// mac intr map register
pub struct INTR_STATUS_REG_1_SPEC;

impl crate::RegisterSpec for INTR_STATUS_REG_1_SPEC {
    type Ux = u32;
}

/// `read()` method returns [`intr_status_reg_1::R`](R) reader structure
impl crate::Readable for INTR_STATUS_REG_1_SPEC {}
