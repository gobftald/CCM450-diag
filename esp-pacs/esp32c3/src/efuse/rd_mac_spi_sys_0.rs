/// Register `RD_MAC_SPI_SYS_0` reader
pub type R = crate::R<RD_MAC_SPI_SYS_0_SPEC>;

/// Field `MAC_0` reader - Stores the low 32 bits of MAC address.
pub type MAC_0_R = crate::FieldReader<u32>;

impl R {
    /// Bits 0:31 - Stores the low 32 bits of MAC address.
    #[inline(always)]
    pub fn mac_0(&self) -> MAC_0_R {
        MAC_0_R::new(self.bits)
    }
}

pub struct RD_MAC_SPI_SYS_0_SPEC;
impl crate::RegisterSpec for RD_MAC_SPI_SYS_0_SPEC {
    type Ux = u32;
}

impl crate::Readable for RD_MAC_SPI_SYS_0_SPEC {}
