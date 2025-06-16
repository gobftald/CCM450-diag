/// Register `INT_ENA` writer
pub type W = crate::W<INT_ENA_SPEC>;

/// Field `T(0-0)` writer - t%s_int_ena
pub type T_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// t(0-0)_int_ena
    ///
    /// warning: `n` is number of field in register. `n == 0` corresponds to `T0` field.
    #[inline(always)]
    pub fn t(&mut self, n: u8) -> T_W<INT_ENA_SPEC> {
        #[allow(clippy::no_effect)]
        [(); 1][n as usize];
        T_W::new(self, n * 0)
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
