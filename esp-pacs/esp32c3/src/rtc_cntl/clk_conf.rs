/// Register `CLK_CONF` writer
pub type W = crate::W<CLK_CONF_SPEC>;

/// Field `CK8M_FORCE_PU` writer - CK8M force power up
pub type CK8M_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    // Bit 26 - CK8M force power up
    #[inline(always)]
    pub fn ck8m_force_pu(&mut self) -> CK8M_FORCE_PU_W<CLK_CONF_SPEC> {
        CK8M_FORCE_PU_W::new(self, 26)
    }
}

pub struct CLK_CONF_SPEC;
impl crate::RegisterSpec for CLK_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for CLK_CONF_SPEC {}

impl crate::Writable for CLK_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
