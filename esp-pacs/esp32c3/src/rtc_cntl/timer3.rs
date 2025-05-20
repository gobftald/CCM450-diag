/// Register `TIMER3` writer
pub type W = crate::W<TIMER3_SPEC>;

/// Field `WIFI_WAIT_TIMER` writer - wifi power domain wakeup time
pub type WIFI_WAIT_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `WIFI_POWERUP_TIMER` writer - wifi power domain power on time
pub type WIFI_POWERUP_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 7>;

/// Field `BT_WAIT_TIMER` writer - bt power domain wakeup time
pub type BT_WAIT_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `BT_POWERUP_TIMER` writer - bt power domain power on time
pub type BT_POWERUP_TIMER_W<'a, REG> = crate::FieldWriter<'a, REG, 7>;

impl W {
    /// Bits 0:8 - wifi power domain wakeup time
    #[inline(always)]
    pub fn wifi_wait_timer(&mut self) -> WIFI_WAIT_TIMER_W<TIMER3_SPEC> {
        WIFI_WAIT_TIMER_W::new(self, 0)
    }

    /// Bits 9:15 - wifi power domain power on time
    #[inline(always)]
    pub fn wifi_powerup_timer(&mut self) -> WIFI_POWERUP_TIMER_W<TIMER3_SPEC> {
        WIFI_POWERUP_TIMER_W::new(self, 9)
    }

    /// Bits 16:24 - bt power domain wakeup time
    #[inline(always)]
    pub fn bt_wait_timer(&mut self) -> BT_WAIT_TIMER_W<TIMER3_SPEC> {
        BT_WAIT_TIMER_W::new(self, 16)
    }

    /// Bits 25:31 - bt power domain power on time
    #[inline(always)]
    pub fn bt_powerup_timer(&mut self) -> BT_POWERUP_TIMER_W<TIMER3_SPEC> {
        BT_POWERUP_TIMER_W::new(self, 25)
    }
}

pub struct TIMER3_SPEC;
impl crate::RegisterSpec for TIMER3_SPEC {
    type Ux = u32;
}

impl crate::Readable for TIMER3_SPEC {}

impl crate::Writable for TIMER3_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
