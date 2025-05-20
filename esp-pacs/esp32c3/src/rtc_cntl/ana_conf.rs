/// Register `ANA_CONF` writer
pub type W = crate::W<ANA_CONF_SPEC>;

/// Field `RESET_POR_FORCE_PD` writer - force no bypass i2c power on reset
pub type RESET_POR_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `PLLA_FORCE_PD` writer - PLLA force power down
pub type PLLA_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `PLLA_FORCE_PU` writer - PLLA force power up
pub type PLLA_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `PVTMON_PU` writer - 1: PVTMON power up
pub type PVTMON_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 18 - force no bypass i2c power on reset
    #[inline(always)]
    pub fn reset_por_force_pd(&mut self) -> RESET_POR_FORCE_PD_W<ANA_CONF_SPEC> {
        RESET_POR_FORCE_PD_W::new(self, 18)
    }

    /// Bit 23 - PLLA force power down
    #[inline(always)]
    pub fn plla_force_pd(&mut self) -> PLLA_FORCE_PD_W<ANA_CONF_SPEC> {
        PLLA_FORCE_PD_W::new(self, 23)
    }

    /// Bit 24 - PLLA force power up
    #[inline(always)]
    pub fn plla_force_pu(&mut self) -> PLLA_FORCE_PU_W<ANA_CONF_SPEC> {
        PLLA_FORCE_PU_W::new(self, 24)
    }

    /// Bit 26 - 1: PVTMON power up
    #[inline(always)]
    pub fn pvtmon_pu(&mut self) -> PVTMON_PU_W<ANA_CONF_SPEC> {
        PVTMON_PU_W::new(self, 26)
    }
}

pub struct ANA_CONF_SPEC;
impl crate::RegisterSpec for ANA_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for ANA_CONF_SPEC {}

impl crate::Writable for ANA_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
