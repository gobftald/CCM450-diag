/// Register `ID` reader
pub type R = crate::R<ID_SPEC>;

// Register `ID` writer
pub type W = crate::W<ID_SPEC>;

/// Field `REG_UPDATE` reader - Software write 1 would synchronize registers into UART Core
/// clock domain and would be cleared by hardware after synchronization is done.
pub type REG_UPDATE_R = crate::BitReader;

/// Field `REG_UPDATE` writer - Software write 1 would synchronize registers into UART Core
/// clock domain and would be cleared by hardware after synchronization is done.
pub type REG_UPDATE_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bit 31 - Software write 1 would synchronize registers into UART Core clock domain
    /// and would be cleared by hardware after synchronization is done."]
    #[inline(always)]
    pub fn reg_update(&self) -> REG_UPDATE_R {
        REG_UPDATE_R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    /// Bit 31 - Software write 1 would synchronize registers into UART Core clock domain
    /// and would be cleared by hardware after synchronization is done.
    #[inline(always)]
    pub fn reg_update(&mut self) -> REG_UPDATE_W<ID_SPEC> {
        REG_UPDATE_W::new(self, 31)
    }
}

pub struct ID_SPEC;
impl crate::RegisterSpec for ID_SPEC {
    type Ux = u32;
}

impl crate::Readable for ID_SPEC {}

impl crate::Writable for ID_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for ID_SPEC {
    const RESET_VALUE: u32 = 0x4000_0500;
}
