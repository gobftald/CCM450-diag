/// Register `CLKDIV` writer
pub type W = crate::W<CLKDIV_SPEC>;

/// Field `CLKDIV` writer - The integral part of the frequency divider factor.
pub type CLKDIV_W<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;

/// Field `FRAG` writer - The decimal part of the frequency divider factor.
pub type FRAG_W<'a, REG> = crate::FieldWriter<'a, REG, 4>;

impl W {
    /// Bits 0:11 - The integral part of the frequency divider factor.
    #[inline(always)]
    pub fn clkdiv(&mut self) -> CLKDIV_W<CLKDIV_SPEC> {
        CLKDIV_W::new(self, 0)
    }

    /// Bits 20:23 - The decimal part of the frequency divider factor.
    #[inline(always)]
    pub fn frag(&mut self) -> FRAG_W<CLKDIV_SPEC> {
        FRAG_W::new(self, 20)
    }
}

pub struct CLKDIV_SPEC;
impl crate::RegisterSpec for CLKDIV_SPEC {
    type Ux = u32;
}

impl crate::Writable for CLKDIV_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for CLKDIV_SPEC {
    const RESET_VALUE: u32 = 0x02b6;
}
