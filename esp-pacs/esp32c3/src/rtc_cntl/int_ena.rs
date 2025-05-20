/// Register `INT_ENA` writer
pub type W = crate::W<INT_ENA_SPEC>;

pub struct INT_ENA_SPEC;
impl crate::RegisterSpec for INT_ENA_SPEC {
    type Ux = u32;
}

impl crate::Writable for INT_ENA_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for INT_ENA_SPEC {
    const RESET_VALUE: u32 = 0;
}
