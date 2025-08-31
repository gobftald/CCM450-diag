/// Register `PIN%s` reader
pub type R = crate::R<PIN_SPEC>;
/// Register `PIN%s` writer
pub type W = crate::W<PIN_SPEC>;

/// Field `PAD_DRIVER` writer - set this bit to select pad driver. 1:open-drain. 0:normal.
pub type PAD_DRIVER_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Interrupt enable bits: bit13: CPU interrupt enabled
/// bit14: CPU non-maskable interrupt enabled.
pub type INT_ENA_R = crate::FieldReader;

impl R {
    /// Interrupt enable bits: bit13: CPU interrupt enabled
    pub fn int_ena(&self) -> INT_ENA_R {
        INT_ENA_R::new(((self.bits >> 13) & 0x1f) as u8)
    }
}

impl W {
    /// Bit 2 - set this bit to select pad driver. 1:open-drain. 0:normal.
    #[inline(always)]
    pub fn pad_driver(&mut self) -> PAD_DRIVER_W<PIN_SPEC> {
        PAD_DRIVER_W::new(self, 2)
    }
}

pub struct PIN_SPEC;
impl crate::RegisterSpec for PIN_SPEC {
    type Ux = u32;
}

impl crate::Readable for PIN_SPEC {}

impl crate::Writable for PIN_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
