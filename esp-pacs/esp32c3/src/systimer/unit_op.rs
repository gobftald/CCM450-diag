/// Register `UNIT%s_OP` reader
pub type R = crate::R<UNIT_OP_SPEC>;

/// Register `UNIT%s_OP` writer
pub type W = crate::W<UNIT_OP_SPEC>;

/// Field `VALUE_VALID` reader - reg_timer_unit0_value_valid
pub type VALUE_VALID_R = crate::BitReader;

/// Field `UPDATE` writer - update timer_unit0
pub type UPDATE_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bit 29 - reg_timer_unit0_value_valid
    #[inline(always)]
    pub fn value_valid(&self) -> VALUE_VALID_R {
        VALUE_VALID_R::new(((self.bits >> 29) & 1) != 0)
    }
}

impl W {
    /// Bit 30 - update timer_unit0
    #[inline(always)]
    pub fn update(&mut self) -> UPDATE_W<UNIT_OP_SPEC> {
        UPDATE_W::new(self, 30)
    }
}

/// SYSTIMER_UNIT%s_OP.
///
/// You can [`read`](crate::Reg::read) this register and get [`unit_op::R`](R).
/// You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write),
/// [`write_with_zero`](crate::Reg::write_with_zero) this register using [`unit_op::W`](W).
/// You can also [`modify`](crate::Reg::modify) this register.
/// See [API](https://docs.rs/svd2rust/#read--modify--write-api).
pub struct UNIT_OP_SPEC;
impl crate::RegisterSpec for UNIT_OP_SPEC {
    type Ux = u32;
}

/// `read()` method returns [`unit_op::R`](R) reader structure
impl crate::Readable for UNIT_OP_SPEC {}

/// `write(|w| ..)` method takes [`unit_op::W`](W) writer structure
impl crate::Writable for UNIT_OP_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

/// `reset()` method sets UNIT%s_OP to value 0
impl crate::Resettable for UNIT_OP_SPEC {
    const RESET_VALUE: u32 = 0;
}
