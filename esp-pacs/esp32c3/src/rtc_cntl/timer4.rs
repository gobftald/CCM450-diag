/// Register `TIMER4` writer
pub type W = crate::W<TIMER4_SPEC>;

/// Field `CPU_TOP_WAIT_TIMER` writer - cpu top power domain wakeup time
pub type CPU_TOP_WAIT_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `CPU_TOP_POWERUP_TIMER` writer - cpu top power domain power on time
pub type CPU_TOP_POWERUP_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 7>;

/// Field `DG_WRAP_WAIT_TIMER` writer - digital wrap power domain wakeup time
pub type DG_WRAP_WAIT_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `DG_WRAP_POWERUP_TIMER` writer - digital wrap power domain power on time
pub type DG_WRAP_POWERUP_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 7>;

impl W {
    /// Bits 0:8 - cpu top power domain wakeup time
    #[inline(always)]
    pub fn cpu_top_wait_timer(&mut self) -> CPU_TOP_WAIT_TIMER_W<TIMER4_SPEC> {
        CPU_TOP_WAIT_TIMER_W::new(self, 0)
    }

    /// Bits 9:15 - cpu top power domain power on time
    #[inline(always)]
    pub fn cpu_top_powerup_timer(&mut self) -> CPU_TOP_POWERUP_TIMER_W<TIMER4_SPEC> {
        CPU_TOP_POWERUP_TIMER_W::new(self, 9)
    }
    /// Bits 16:24 - digital wrap power domain wakeup time
    #[inline(always)]
    pub fn dg_wrap_wait_timer(&mut self) -> DG_WRAP_WAIT_TIMER_W<TIMER4_SPEC> {
        DG_WRAP_WAIT_TIMER_W::new(self, 16)
    }

    /// Bits 25:31 - digital wrap power domain power on time
    #[inline(always)]
    pub fn dg_wrap_powerup_timer(&mut self) -> DG_WRAP_POWERUP_TIMER_W<TIMER4_SPEC> {
        DG_WRAP_POWERUP_TIMER_W::new(self, 25)
    }
}

pub struct TIMER4_SPEC;
impl crate::RegisterSpec for TIMER4_SPEC {
    type Ux = u32;
}

impl crate::Readable for TIMER4_SPEC {}

impl crate::Writable for TIMER4_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
