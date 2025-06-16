/// Register `ALARMHI` writer
pub type W = crate::W<ALARMHI_SPEC>;

/// Field `ALARM_HI` writer - reg_t0_alarm_hi.
pub type ALARM_HI_W<'a, REG> = crate::FieldWriter<'a, REG, 22, u32>;

impl W {
    /// Bits 0:21 - reg_t0_alarm_hi.
    #[inline(always)]
    pub fn alarm_hi(&mut self) -> ALARM_HI_W<ALARMHI_SPEC> {
        ALARM_HI_W::new(self, 0)
    }
}

pub struct ALARMHI_SPEC;
impl crate::RegisterSpec for ALARMHI_SPEC {
    type Ux = u32;
}

impl crate::Writable for ALARMHI_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for ALARMHI_SPEC {
    const RESET_VALUE: u32 = 0;
}
