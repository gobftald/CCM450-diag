/// Register `MEM_PD_MASK` writer
pub type W = crate::W<MEM_PD_MASK_SPEC>;

/// Field `LSLP_MEM_PD_MASK` writer - reg_lslp_mem_pd_mask
pub type LSLP_MEM_PD_MASK_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 0 - reg_lslp_mem_pd_mask
    #[inline(always)]
    pub fn lslp_mem_pd_mask(&mut self) -> LSLP_MEM_PD_MASK_W<MEM_PD_MASK_SPEC> {
        LSLP_MEM_PD_MASK_W::new(self, 0)
    }
}

pub struct MEM_PD_MASK_SPEC;
impl crate::RegisterSpec for MEM_PD_MASK_SPEC {
    type Ux = u32;
}

impl crate::Readable for MEM_PD_MASK_SPEC {}

impl crate::Writable for MEM_PD_MASK_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
