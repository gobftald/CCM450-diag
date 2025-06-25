/// Register `WIFI_CLK_EN` reader
pub type R = crate::R<WIFI_CLK_EN_SPEC>;
/// Register `WIFI_CLK_EN` writer
pub type W = crate::W<WIFI_CLK_EN_SPEC>;

/// Field `WIFI_CLK_EN` reader - reg_wifi_clk_en
pub type WIFI_CLK_EN_R = crate::FieldReader<u32>;
/// Field `WIFI_CLK_EN` writer - reg_wifi_clk_en
pub type WIFI_CLK_EN_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl R {
    /// Bits 0:31 - reg_wifi_clk_en
    #[inline(always)]
    pub fn wifi_clk_en(&self) -> WIFI_CLK_EN_R {
        WIFI_CLK_EN_R::new(self.bits)
    }
}

impl W {
    /// Bits 0:31 - reg_wifi_clk_en
    #[inline(always)]
    pub fn wifi_clk_en(&mut self) -> WIFI_CLK_EN_W<WIFI_CLK_EN_SPEC> {
        WIFI_CLK_EN_W::new(self, 0)
    }
}

pub struct WIFI_CLK_EN_SPEC;
impl crate::RegisterSpec for WIFI_CLK_EN_SPEC {
    type Ux = u32;
}

impl crate::Readable for WIFI_CLK_EN_SPEC {}

impl crate::Writable for WIFI_CLK_EN_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
