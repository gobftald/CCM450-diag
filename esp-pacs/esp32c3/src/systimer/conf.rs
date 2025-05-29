/// Register `CONF` reader
pub type R = crate::R<CONF_SPEC>;
/// Register `CONF` writer
pub type W = crate::W<CONF_SPEC>;

/// Field `TARGET2_WORK_EN` reader - target2 work enable
pub type TARGET2_WORK_EN_R = crate::BitReader;
/// Field `TARGET2_WORK_EN` writer - target2 work enable
pub type TARGET2_WORK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TARGET1_WORK_EN` reader - target1 work enable
pub type TARGET1_WORK_EN_R = crate::BitReader;
/// Field `TARGET1_WORK_EN` writer - target1 work enable
pub type TARGET1_WORK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TARGET0_WORK_EN` reader - target0 work enable
pub type TARGET0_WORK_EN_R = crate::BitReader;
/// Field `TARGET0_WORK_EN` writer - target0 work enable
pub type TARGET0_WORK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TIMER_UNIT0_CORE0_STALL_EN` writer - If timer unit0 is stalled when core0 stalled
pub type TIMER_UNIT0_CORE0_STALL_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bit 22 - target2 work enable
    #[inline(always)]
    pub fn target2_work_en(&self) -> TARGET2_WORK_EN_R {
        TARGET2_WORK_EN_R::new(((self.bits >> 22) & 1) != 0)
    }

    /// Bit 23 - target1 work enable
    #[inline(always)]
    pub fn target1_work_en(&self) -> TARGET1_WORK_EN_R {
        TARGET1_WORK_EN_R::new(((self.bits >> 23) & 1) != 0)
    }

    /// Bit 24 - target0 work enable
    #[inline(always)]
    pub fn target0_work_en(&self) -> TARGET0_WORK_EN_R {
        TARGET0_WORK_EN_R::new(((self.bits >> 24) & 1) != 0)
    }
}

impl W {
    /// Bit 22 - target2 work enable
    #[inline(always)]
    pub fn target2_work_en(&mut self) -> TARGET2_WORK_EN_W<CONF_SPEC> {
        TARGET2_WORK_EN_W::new(self, 22)
    }

    /// Bit 23 - target1 work enable
    #[inline(always)]
    pub fn target1_work_en(&mut self) -> TARGET1_WORK_EN_W<CONF_SPEC> {
        TARGET1_WORK_EN_W::new(self, 23)
    }

    /// Bit 24 - target0 work enable
    #[inline(always)]
    pub fn target0_work_en(&mut self) -> TARGET0_WORK_EN_W<CONF_SPEC> {
        TARGET0_WORK_EN_W::new(self, 24)
    }

    /// Bit 28 - If timer unit0 is stalled when core0 stalled
    #[inline(always)]
    pub fn timer_unit0_core0_stall_en(&mut self) -> TIMER_UNIT0_CORE0_STALL_EN_W<CONF_SPEC> {
        TIMER_UNIT0_CORE0_STALL_EN_W::new(self, 28)
    }
}

pub struct CONF_SPEC;
impl crate::RegisterSpec for CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for CONF_SPEC {}

impl crate::Writable for CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
