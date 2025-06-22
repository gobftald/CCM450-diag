/// Register `CPU_INTR_FROM_CPU_2` writer
pub type W = crate::W<CPU_INTR_FROM_CPU_2_SPEC>;

/// Field `CPU_INTR_FROM_CPU_2` writer - reg_cpu_intr_from_cpu_2
pub type CPU_INTR_FROM_CPU_2_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 0 - reg_cpu_intr_from_cpu_2
    #[inline(always)]
    pub fn cpu_intr_from_cpu_2(&mut self) -> CPU_INTR_FROM_CPU_2_W<CPU_INTR_FROM_CPU_2_SPEC> {
        CPU_INTR_FROM_CPU_2_W::new(self, 0)
    }
}

pub struct CPU_INTR_FROM_CPU_2_SPEC;
impl crate::RegisterSpec for CPU_INTR_FROM_CPU_2_SPEC {
    type Ux = u32;
}

impl crate::Readable for CPU_INTR_FROM_CPU_2_SPEC {}

impl crate::Writable for CPU_INTR_FROM_CPU_2_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
