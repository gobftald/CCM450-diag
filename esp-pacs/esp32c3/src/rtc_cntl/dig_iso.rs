/// Register `DIG_ISO` writer
pub type W = crate::W<DIG_ISO_SPEC>;

/// Field `DG_PAD_FORCE_NOISO` writer - digital pad force no ISO
pub type DG_PAD_FORCE_NOISO_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DG_PAD_FORCE_UNHOLD` writer - digital pad force un-hold
pub type DG_PAD_FORCE_UNHOLD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BT_FORCE_NOISO` writer - bt force no ISO
pub type BT_FORCE_NOISO_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DG_PERI_FORCE_NOISO` writer - digital peri force no ISO
pub type DG_PERI_FORCE_NOISO_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `CPU_TOP_FORCE_NOISO` writer - cpu force no ISO
pub type CPU_TOP_FORCE_NOISO_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `WIFI_FORCE_NOISO` writer - wifi force no ISO
pub type WIFI_FORCE_NOISO_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DG_WRAP_FORCE_NOISO` writer - digital core force no ISO
pub type DG_WRAP_FORCE_NOISO_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 12 - digital pad force no ISO
    #[inline(always)]
    pub fn dg_pad_force_noiso(&mut self) -> DG_PAD_FORCE_NOISO_W<DIG_ISO_SPEC> {
        DG_PAD_FORCE_NOISO_W::new(self, 12)
    }

    /// Bit 14 - digital pad force un-hold
    #[inline(always)]
    pub fn dg_pad_force_unhold(&mut self) -> DG_PAD_FORCE_UNHOLD_W<DIG_ISO_SPEC> {
        DG_PAD_FORCE_UNHOLD_W::new(self, 14)
    }

    /// Bit 23 - bt force no ISO
    #[inline(always)]
    pub fn bt_force_noiso(&mut self) -> BT_FORCE_NOISO_W<DIG_ISO_SPEC> {
        BT_FORCE_NOISO_W::new(self, 23)
    }

    /// Bit 25 - digital peri force no ISO
    #[inline(always)]
    pub fn dg_peri_force_noiso(&mut self) -> DG_PERI_FORCE_NOISO_W<DIG_ISO_SPEC> {
        DG_PERI_FORCE_NOISO_W::new(self, 25)
    }

    /// Bit 27 - cpu force no ISO
    #[inline(always)]
    pub fn cpu_top_force_noiso(&mut self) -> CPU_TOP_FORCE_NOISO_W<DIG_ISO_SPEC> {
        CPU_TOP_FORCE_NOISO_W::new(self, 27)
    }

    /// Bit 29 - wifi force no ISO
    #[inline(always)]
    pub fn wifi_force_noiso(&mut self) -> WIFI_FORCE_NOISO_W<DIG_ISO_SPEC> {
        WIFI_FORCE_NOISO_W::new(self, 29)
    }

    /// Bit 31 - digital core force no ISO
    #[inline(always)]
    pub fn dg_wrap_force_noiso(&mut self) -> DG_WRAP_FORCE_NOISO_W<DIG_ISO_SPEC> {
        DG_WRAP_FORCE_NOISO_W::new(self, 31)
    }
}

pub struct DIG_ISO_SPEC;
impl crate::RegisterSpec for DIG_ISO_SPEC {
    type Ux = u32;
}

impl crate::Readable for DIG_ISO_SPEC {}

impl crate::Writable for DIG_ISO_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
