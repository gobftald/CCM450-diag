/// Register `CPU_INT_CLEAR` writer
pub type W = crate::W<CPU_INT_CLEAR_SPEC>;

/// Field `CPU_INT_CLEAR` writer - reg_core0_cpu_int_clear
pub type CPU_INT_CLEAR_W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;

impl W {
    /// Bits 0:31 - reg_core0_cpu_int_clear
    #[inline(always)]
    pub fn cpu_int_clear(&mut self) -> CPU_INT_CLEAR_W<CPU_INT_CLEAR_SPEC> {
        CPU_INT_CLEAR_W::new(self, 0)
    }
}

pub struct CPU_INT_CLEAR_SPEC;
impl crate::RegisterSpec for CPU_INT_CLEAR_SPEC {
    type Ux = u32;
}

impl crate::Writable for CPU_INT_CLEAR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for CPU_INT_CLEAR_SPEC {
    const RESET_VALUE: u32 = 0;
}
