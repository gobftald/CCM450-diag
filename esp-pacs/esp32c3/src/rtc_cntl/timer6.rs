/// Register `TIMER6` writer
pub type W = crate::W<TIMER6_SPEC>;

/// Field `DG_PERI_WAIT_TIMER` writer - digital peri power domain wakeup time
pub type DG_PERI_WAIT_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `DG_PERI_POWERUP_TIMER` writer - digital peri power domain power on time
pub type DG_PERI_POWERUP_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 7>;

impl W {
    /// Bits 16:24 - digital peri power domain wakeup time
    #[inline(always)]
    pub fn dg_peri_wait_timer(&mut self) -> DG_PERI_WAIT_TIMER_W<TIMER6_SPEC> {
        DG_PERI_WAIT_TIMER_W::new(self, 16)
    }

    /// Bits 25:31 - digital peri power domain power on time
    #[inline(always)]
    pub fn dg_peri_powerup_timer(&mut self) -> DG_PERI_POWERUP_TIMER_W<TIMER6_SPEC> {
        DG_PERI_POWERUP_TIMER_W::new(self, 25)
    }
}

pub struct TIMER6_SPEC;
impl crate::RegisterSpec for TIMER6_SPEC {
    type Ux = u32;
}

impl crate::Readable for TIMER6_SPEC {}

impl crate::Writable for TIMER6_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
