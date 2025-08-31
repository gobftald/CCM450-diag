/// Register `PERIP_RST_EN0` writer
pub type W = crate::W<PERIP_RST_EN0_SPEC>;

/// Field `UART_RST` writer - reg_uart_rst"]
pub type UART_RST_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `SYSTIMER_RST` writer - reg_systimer_rst
pub type SYSTIMER_RST_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 2 - reg_uart_rst"]
    #[inline(always)]
    pub fn uart_rst(&mut self) -> UART_RST_W<PERIP_RST_EN0_SPEC> {
        UART_RST_W::new(self, 2)
    }

    /// Bit 29 - reg_systimer_rst
    #[inline(always)]
    pub fn systimer_rst(&mut self) -> SYSTIMER_RST_W<PERIP_RST_EN0_SPEC> {
        SYSTIMER_RST_W::new(self, 29)
    }
}

pub struct PERIP_RST_EN0_SPEC;
impl crate::RegisterSpec for PERIP_RST_EN0_SPEC {
    type Ux = u32;
}

impl crate::Readable for PERIP_RST_EN0_SPEC {}

impl crate::Writable for PERIP_RST_EN0_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
