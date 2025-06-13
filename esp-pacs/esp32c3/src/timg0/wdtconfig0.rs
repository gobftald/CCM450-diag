/// Register `WDTCONFIG0` writer
pub type W = crate::W<WDTCONFIG0_SPEC>;

/// Field `WDT_USE_XTAL` writer - reg_wdt_use_xtal.
pub type WDT_USE_XTAL_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 21 - reg_wdt_use_xtal.
    #[inline(always)]
    pub fn wdt_use_xtal(&mut self) -> WDT_USE_XTAL_W<WDTCONFIG0_SPEC> {
        WDT_USE_XTAL_W::new(self, 21)
    }
}

pub struct WDTCONFIG0_SPEC;
impl crate::RegisterSpec for WDTCONFIG0_SPEC {
    type Ux = u32;
}

impl crate::Readable for WDTCONFIG0_SPEC {}

impl crate::Writable for WDTCONFIG0_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for WDTCONFIG0_SPEC {
    const RESET_VALUE: u32 = 0x0004_c000;
}
