/// Register `CPU_INT_PRI%s` writer
pub type W = crate::W<CPU_INT_PRI_SPEC>;

/// Field `MAP` writer - reg_core0_cpu_pri_0_map
pub type MAP_W<'a, REG> = crate::FieldWriter<'a, REG, 4>;

impl W {
    /// Bits 0:3 - reg_core0_cpu_pri_0_map
    #[inline(always)]
    #[must_use]
    pub fn map(&mut self) -> MAP_W<CPU_INT_PRI_SPEC> {
        MAP_W::new(self, 0)
    }
}

/// mac intr map register
pub struct CPU_INT_PRI_SPEC;

impl crate::RegisterSpec for CPU_INT_PRI_SPEC {
    type Ux = u32;
}

/// `write(|w| ..)` method takes [`cpu_int_pri::W`](W) writer structure
impl crate::Writable for CPU_INT_PRI_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

/// `reset()` method sets CPU_INT_PRI%s to value 0
impl crate::Resettable for CPU_INT_PRI_SPEC {
    const RESET_VALUE: u32 = 0;
}
