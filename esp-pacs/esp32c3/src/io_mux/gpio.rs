/// Register `GPIO%s` writer
pub type W = crate::W<GPIO_SPEC>;

/// Field `SLP_SEL` writer - Sleep mode selection of this pad. Set to 1 to put the pad in pad mode.
pub type SLP_SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `FUN_WPD` writer - Pull-down enable of the pad.
/// 1: internal pull-down enabled; 0: internal pull-down disabled.
pub type FUN_WPD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `FUN_WPU` writer - Pull-up enable of the pad. 1: internal pull-up enabled; 0: internal pull-up disabled.
pub type FUN_WPU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `FUN_IE` writer - Input enable of the pad. 1: input enabled; 0: input disabled.
pub type FUN_IE_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `FUN_DRV` writer - Select the drive strength of the pad. 0: ~5 mA; 1: ~10mA; 2: ~20mA; 3: ~40mA."]
pub type FUN_DRV_W<'a, REG> = crate::FieldWriter<'a, REG, 2>;

/// Field `MCU_SEL` writer - Select IO MUX function for this signal.
/// 0: Select Function 1; 1: Select Function 2; etc.
pub type MCU_SEL_W<'a, REG> = crate::FieldWriter<'a, REG, 3>;

impl W {
    /// Bit 1 - Sleep mode selection of this pad. Set to 1 to put the pad in pad mode.
    #[inline(always)]
    pub fn slp_sel(&mut self) -> SLP_SEL_W<GPIO_SPEC> {
        SLP_SEL_W::new(self, 1)
    }

    /// Bit 7 - Pull-down enable of the pad. 1: internal pull-down enabled; 0: internal pull-down disabled.
    #[inline(always)]
    pub fn fun_wpd(&mut self) -> FUN_WPD_W<GPIO_SPEC> {
        FUN_WPD_W::new(self, 7)
    }

    /// Bit 8 - Pull-up enable of the pad. 1: internal pull-up enabled; 0: internal pull-up disabled.
    #[inline(always)]
    pub fn fun_wpu(&mut self) -> FUN_WPU_W<GPIO_SPEC> {
        FUN_WPU_W::new(self, 8)
    }

    /// Bit 9 - Input enable of the pad. 1: input enabled; 0: input disabled.
    #[inline(always)]
    pub fn fun_ie(&mut self) -> FUN_IE_W<GPIO_SPEC> {
        FUN_IE_W::new(self, 9)
    }

    /// Bits 10:11 - Select the drive strength of the pad. 0: ~5 mA; 1: ~10mA; 2: ~20mA; 3: ~40mA.
    #[inline(always)]
    pub fn fun_drv(&mut self) -> FUN_DRV_W<GPIO_SPEC> {
        FUN_DRV_W::new(self, 10)
    }

    /// Bits 12:14 - Select IO MUX function for this signal. 0: Select Function 1; 1: Select Function 2; etc.
    #[inline(always)]
    pub fn mcu_sel(&mut self) -> MCU_SEL_W<GPIO_SPEC> {
        MCU_SEL_W::new(self, 12)
    }
}

pub struct GPIO_SPEC;
impl crate::RegisterSpec for GPIO_SPEC {
    type Ux = u32;
}

impl crate::Readable for GPIO_SPEC {}

impl crate::Writable for GPIO_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
