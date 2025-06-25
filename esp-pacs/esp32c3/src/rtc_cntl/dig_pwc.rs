/// Register `DIG_PWC` writer
pub type W = crate::W<DIG_PWC_SPEC>;

/// Field `BT_FORCE_PD` writer - bt force power down
pub type BT_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BT_FORCE_PU` writer - bt force power up
pub type BT_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DG_PERI_FORCE_PU` writer - digital peri force power up
pub type DG_PERI_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `WIFI_FORCE_PD` writer - wifi force power down
pub type WIFI_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `WIFI_FORCE_PU` writer - wifi force power up
pub type WIFI_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DG_WRAP_FORCE_PU` writer - digital core force power up
pub type DG_WRAP_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `CPU_TOP_FORCE_PU` writer - cpu force power up
pub type CPU_TOP_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 11 - bt force power down
    #[inline(always)]
    pub fn bt_force_pd(&mut self) -> BT_FORCE_PD_W<DIG_PWC_SPEC> {
        BT_FORCE_PD_W::new(self, 11)
    }

    /// Bit 12 - bt force power up
    #[inline(always)]
    pub fn bt_force_pu(&mut self) -> BT_FORCE_PU_W<DIG_PWC_SPEC> {
        BT_FORCE_PU_W::new(self, 12)
    }

    /// Bit 14 - digital peri force power up
    #[inline(always)]
    pub fn dg_peri_force_pu(&mut self) -> DG_PERI_FORCE_PU_W<DIG_PWC_SPEC> {
        DG_PERI_FORCE_PU_W::new(self, 14)
    }

    /// Bit 17 - wifi force power down
    #[inline(always)]
    pub fn wifi_force_pd(&mut self) -> WIFI_FORCE_PD_W<DIG_PWC_SPEC> {
        WIFI_FORCE_PD_W::new(self, 17)
    }

    /// Bit 18 - wifi force power up
    #[inline(always)]
    pub fn wifi_force_pu(&mut self) -> WIFI_FORCE_PU_W<DIG_PWC_SPEC> {
        WIFI_FORCE_PU_W::new(self, 18)
    }

    /// Bit 20 - digital core force power up
    #[inline(always)]
    pub fn dg_wrap_force_pu(&mut self) -> DG_WRAP_FORCE_PU_W<DIG_PWC_SPEC> {
        DG_WRAP_FORCE_PU_W::new(self, 20)
    }

    /// Bit 22 - cpu force power up
    #[inline(always)]
    pub fn cpu_top_force_pu(&mut self) -> CPU_TOP_FORCE_PU_W<DIG_PWC_SPEC> {
        CPU_TOP_FORCE_PU_W::new(self, 22)
    }
}

pub struct DIG_PWC_SPEC;
impl crate::RegisterSpec for DIG_PWC_SPEC {
    type Ux = u32;
}

impl crate::Readable for DIG_PWC_SPEC {}

impl crate::Writable for DIG_PWC_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
