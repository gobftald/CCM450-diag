/// Register `CPU_INT_THRESH` reader
pub type R = crate::R<CPU_INT_THRESH_SPEC>;
/// Register `CPU_INT_THRESH` writer
pub type W = crate::W<CPU_INT_THRESH_SPEC>;

/// Field `CPU_INT_THRESH` reader - reg_core0_cpu_int_thresh
pub type CPU_INT_THRESH_R = crate::FieldReader;
/// Field `CPU_INT_THRESH` writer - reg_core0_cpu_int_thresh
pub type CPU_INT_THRESH_W<'a, REG> = crate::FieldWriter<'a, REG, 4>;

impl R {
    /// Bits 0:3 - reg_core0_cpu_int_thresh
    #[inline(always)]
    pub fn cpu_int_thresh(&self) -> CPU_INT_THRESH_R {
        CPU_INT_THRESH_R::new((self.bits & 0x0f) as u8)
    }
}

impl W {
    /// Bits 0:3 - reg_core0_cpu_int_thresh
    #[inline(always)]
    pub fn cpu_int_thresh(&mut self) -> CPU_INT_THRESH_W<CPU_INT_THRESH_SPEC> {
        CPU_INT_THRESH_W::new(self, 0)
    }
}

pub struct CPU_INT_THRESH_SPEC;
impl crate::RegisterSpec for CPU_INT_THRESH_SPEC {
    type Ux = u32;
}

impl crate::Readable for CPU_INT_THRESH_SPEC {}

impl crate::Writable for CPU_INT_THRESH_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for CPU_INT_THRESH_SPEC {
    const RESET_VALUE: u32 = 0;
}
