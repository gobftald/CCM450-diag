/// Register `RTC_CNTL` writer
pub type W = crate::W<RTC_CNTL_SPEC>;

/// Field `DBOOST_FORCE_PD` writer - RTC_DBOOST force power down
pub type DBOOST_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `DBOOST_FORCE_PU` writer - RTC_DBOOST force power up
pub type DBOOST_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `REGULATOR_FORCE_PU` writer - RTC_REG force power up
pub type REGULATOR_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 28 - RTC_DBOOST force power down
    #[inline(always)]
    pub fn dboost_force_pd(&mut self) -> DBOOST_FORCE_PD_W<RTC_CNTL_SPEC> {
        DBOOST_FORCE_PD_W::new(self, 28)
    }

    /// Bit 29 - RTC_DBOOST force power up
    #[inline(always)]
    pub fn dboost_force_pu(&mut self) -> DBOOST_FORCE_PU_W<RTC_CNTL_SPEC> {
        DBOOST_FORCE_PU_W::new(self, 29)
    }

    /// Bit 31 - RTC_REG force power up
    #[inline(always)]
    pub fn regulator_force_pu(&mut self) -> REGULATOR_FORCE_PU_W<RTC_CNTL_SPEC> {
        REGULATOR_FORCE_PU_W::new(self, 31)
    }
}

pub struct RTC_CNTL_SPEC;
impl crate::RegisterSpec for RTC_CNTL_SPEC {
    type Ux = u32;
}

impl crate::Readable for RTC_CNTL_SPEC {}

impl crate::Writable for RTC_CNTL_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
