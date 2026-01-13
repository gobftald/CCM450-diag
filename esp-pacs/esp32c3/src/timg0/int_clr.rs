/// Register `INT_CLR` writer
pub type W = crate::W<INT_CLR_SPEC>;

/// Field `T(0-0)` writer - t%s_int_clr
pub type T_W<'a, REG> = crate::BitWriter1C<'a, REG>;

impl W {
    /// t(0-0)_int_clr
    #[inline(always)]
    pub fn t(&mut self, n: u8) -> T_W<'_, INT_CLR_SPEC> {
        #[allow(clippy::no_effect)]
        [(); 1][n as usize];
        T_W::new(self, 0)
    }
}

pub struct INT_CLR_SPEC;
impl crate::RegisterSpec for INT_CLR_SPEC {
    type Ux = u32;
}

impl crate::Writable for INT_CLR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x03;
}

impl crate::Resettable for INT_CLR_SPEC {
    const RESET_VALUE: u32 = 0;
}
