/// Register `INT_CLR` writer
pub type W = crate::W<INT_CLR_SPEC>;

/// Field `TARGET(0-2)` writer - interupt%s clear
pub type TARGET_W<'a, REG> = crate::BitWriter1C<'a, REG>;

impl W {
    /// interupt(0-2) clear
    ///
    /// `n` is number of field in register. `n == 0` corresponds to `TARGET0` field
    #[inline(always)]
    pub fn target(&mut self, n: u8) -> TARGET_W<INT_CLR_SPEC> {
        #[allow(clippy::no_effect)]
        [(); 3][n as usize];
        TARGET_W::new(self, n)
    }
}

pub struct INT_CLR_SPEC;
impl crate::RegisterSpec for INT_CLR_SPEC {
    type Ux = u32;
}

impl crate::Writable for INT_CLR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x07;
}

impl crate::Resettable for INT_CLR_SPEC {
    const RESET_VALUE: u32 = 0;
}
