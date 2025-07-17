/// Register `WIFI_RST_EN` writer
pub type W = crate::W<WIFI_RST_EN_SPEC>;

/// Field `MAC_RST` writer - Set this bit to reset MAC module. Clear the bit to release MAC module.
pub type MAC_RST_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    // Bit 2 - Set this bit to reset MAC module. Clear the bit to release MAC module.
    #[inline(always)]
    pub fn mac_rst(&mut self) -> MAC_RST_W<WIFI_RST_EN_SPEC> {
        MAC_RST_W::new(self, 2)
    }
}

/// APB_CTRL_WIFI_RST_EN_REG
pub struct WIFI_RST_EN_SPEC;
impl crate::RegisterSpec for WIFI_RST_EN_SPEC {
    type Ux = u32;
}

impl crate::Readable for WIFI_RST_EN_SPEC {}

impl crate::Writable for WIFI_RST_EN_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
