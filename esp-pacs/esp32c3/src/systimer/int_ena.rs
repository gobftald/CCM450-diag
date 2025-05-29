/// Register `INT_ENA` writer
pub type W = crate::W<INT_ENA_SPEC>;

/// Field `TARGET(0-2)` writer - interupt%s enable
pub type TARGET_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// interupt(0-2) enable
    ///
    /// `n` is number of field in register. `n == 0` corresponds to `TARGET0`
    #[inline(always)]
    pub fn target(&mut self, n: u8) -> TARGET_W<INT_ENA_SPEC> {
        #[allow(clippy::no_effect)]
        [(); 3][n as usize];
        TARGET_W::new(self, n)
    }
}

pub struct INT_ENA_SPEC;
impl crate::RegisterSpec for INT_ENA_SPEC {
    type Ux = u32;
}

impl crate::Readable for INT_ENA_SPEC {}

impl crate::Writable for INT_ENA_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
