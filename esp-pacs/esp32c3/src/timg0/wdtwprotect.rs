/// Register `WDTWPROTECT` writer
pub type W = crate::W<WDTWPROTECT_SPEC>;

/// Field `WDT_WKEY` writer - reg_wdt_wkey.
pub type WDT_WKEY_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - reg_wdt_wkey.
    #[inline(always)]
    pub fn wdt_wkey(&mut self) -> WDT_WKEY_W<WDTWPROTECT_SPEC> {
        WDT_WKEY_W::new(self, 0)
    }
}

pub struct WDTWPROTECT_SPEC;
impl crate::RegisterSpec for WDTWPROTECT_SPEC {
    type Ux = u32;
}

impl crate::Writable for WDTWPROTECT_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for WDTWPROTECT_SPEC {
    const RESET_VALUE: u32 = 0x50d8_3aa1;
}
