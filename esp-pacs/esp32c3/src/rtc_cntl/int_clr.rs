/// Register `INT_CLR` writer
pub type W = crate::W<INT_CLR_SPEC>;

pub struct INT_CLR_SPEC;
impl crate::RegisterSpec for INT_CLR_SPEC {
    type Ux = u32;
}

impl crate::Writable for INT_CLR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0019_860b;
}

impl crate::Resettable for INT_CLR_SPEC {
    const RESET_VALUE: u32 = 0;
}
