/// Register `CLK_CONF` writer
pub type W = crate::W<CLK_CONF_SPEC>;

/// Field `SCLK_DIV_B` writer - The denominator of the frequency divider factor.
pub type SCLK_DIV_B_W<'a, REG> = crate::FieldWriter<'a, REG, 6>;

/// Field `SCLK_DIV_A` writer - The numerator of the frequency divider factor.
pub type SCLK_DIV_A_W<'a, REG> = crate::FieldWriter<'a, REG, 6>;

/// Field `SCLK_DIV_NUM` writer - The integral part of the frequency divider factor.
pub type SCLK_DIV_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

/// Field `SCLK_SEL` writer - UART clock source select. 1: 80Mhz, 2: 8Mhz, 3: XTAL.
pub type SCLK_SEL_W<'a, REG> = crate::FieldWriter<'a, REG, 2>;

/// Field `SCLK_EN` writer - Set this bit to enable UART Tx/Rx clock.
pub type SCLK_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `RST_CORE` writer - Write 1 then write 0 to this bit, reset UART Tx/Rx.
pub type RST_CORE_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bits 0:5 - The denominator of the frequency divider factor.
    #[inline(always)]
    pub fn sclk_div_b(&mut self) -> SCLK_DIV_B_W<CLK_CONF_SPEC> {
        SCLK_DIV_B_W::new(self, 0)
    }

    /// Bits 6:11 - The numerator of the frequency divider factor.
    #[inline(always)]
    pub fn sclk_div_a(&mut self) -> SCLK_DIV_A_W<CLK_CONF_SPEC> {
        SCLK_DIV_A_W::new(self, 6)
    }

    /// Bits 12:19 - The integral part of the frequency divider factor.
    #[inline(always)]
    pub fn sclk_div_num(&mut self) -> SCLK_DIV_NUM_W<CLK_CONF_SPEC> {
        SCLK_DIV_NUM_W::new(self, 12)
    }

    /// Bits 20:21 - UART clock source select. 1: 80Mhz, 2: 8Mhz, 3: XTAL.
    #[inline(always)]
    pub fn sclk_sel(&mut self) -> SCLK_SEL_W<CLK_CONF_SPEC> {
        SCLK_SEL_W::new(self, 20)
    }

    /// Bit 22 - Set this bit to enable UART Tx/Rx clock.
    #[inline(always)]
    pub fn sclk_en(&mut self) -> SCLK_EN_W<CLK_CONF_SPEC> {
        SCLK_EN_W::new(self, 22)
    }

    /// Bit 23 - Write 1 then write 0 to this bit, reset UART Tx/Rx.
    #[inline(always)]
    pub fn rst_core(&mut self) -> RST_CORE_W<CLK_CONF_SPEC> {
        RST_CORE_W::new(self, 23)
    }
}

pub struct CLK_CONF_SPEC;
impl crate::RegisterSpec for CLK_CONF_SPEC {
    type Ux = u32;
}
impl crate::Readable for CLK_CONF_SPEC {}

impl crate::Writable for CLK_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for CLK_CONF_SPEC {
    const RESET_VALUE: u32 = 0x0370_1000;
}
