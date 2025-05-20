/// Register `SWD_WPROTECT` writer
pub type W = crate::W<SWD_WPROTECT_SPEC>;

/// Field `SWD_WKEY` writer - the key of super wdt"
pub type SWD_WKEY_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - the key of super wdt
    #[inline(always)]
    pub fn swd_wkey(&mut self) -> SWD_WKEY_W<SWD_WPROTECT_SPEC> {
        SWD_WKEY_W::new(self, 0)
    }
}

pub struct SWD_WPROTECT_SPEC;
impl crate::RegisterSpec for SWD_WPROTECT_SPEC {
    type Ux = u32;
}

impl crate::Writable for SWD_WPROTECT_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for SWD_WPROTECT_SPEC {
    const RESET_VALUE: u32 = 0;
}
