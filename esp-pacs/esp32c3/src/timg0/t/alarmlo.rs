/// Register `ALARMLO` writer
pub type W = crate::W<ALARMLO_SPEC>;

/// Field `ALARM_LO` writer - reg_t0_alarm_lo.
pub type ALARM_LO_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - reg_t0_alarm_lo.
    #[inline(always)]
    pub fn alarm_lo(&mut self) -> ALARM_LO_W<ALARMLO_SPEC> {
        ALARM_LO_W::new(self, 0)
    }
}

pub struct ALARMLO_SPEC;
impl crate::RegisterSpec for ALARMLO_SPEC {
    type Ux = u32;
}

impl crate::Writable for ALARMLO_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for ALARMLO_SPEC {
    const RESET_VALUE: u32 = 0;
}
