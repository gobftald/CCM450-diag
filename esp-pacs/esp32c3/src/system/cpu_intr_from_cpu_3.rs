/// Register `CPU_INTR_FROM_CPU_3` writer
pub type W = crate::W<CPU_INTR_FROM_CPU_3_SPEC>;

/// Field `CPU_INTR_FROM_CPU_3` writer - reg_cpu_intr_from_cpu_3
pub type CPU_INTR_FROM_CPU_3_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    #[doc = "Bit 0 - reg_cpu_intr_from_cpu_3"]
    #[inline(always)]
    pub fn cpu_intr_from_cpu_3(&mut self) -> CPU_INTR_FROM_CPU_3_W<CPU_INTR_FROM_CPU_3_SPEC> {
        CPU_INTR_FROM_CPU_3_W::new(self, 0)
    }
}

pub struct CPU_INTR_FROM_CPU_3_SPEC;
impl crate::RegisterSpec for CPU_INTR_FROM_CPU_3_SPEC {
    type Ux = u32;
}

impl crate::Writable for CPU_INTR_FROM_CPU_3_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for CPU_INTR_FROM_CPU_3_SPEC {
    const RESET_VALUE: u32 = 0;
}
