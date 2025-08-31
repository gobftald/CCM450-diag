#[doc = "Register `CONF0` reader"]
pub type R = crate::R<CONF0_SPEC>;

// Register `CONF0` writer
pub type W = crate::W<CONF0_SPEC>;

/// Field `PARITY` writer - This register is used to configure the parity check mode.
pub type PARITY_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `PARITY_EN` reader - Set this bit to enable uart parity check.
pub type PARITY_EN_R = crate::BitReader;
/// Field `PARITY_EN` writer - Set this bit to enable uart parity check.
pub type PARITY_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BIT_NUM` reader - This register is used to set the length of data.
pub type BIT_NUM_R = crate::FieldReader;
/// Field `BIT_NUM` writer - This register is used to set the length of data.
pub type BIT_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 2>;

/// Field `STOP_BIT_NUM` reader - This register is used to set the length of stop bit.
pub type STOP_BIT_NUM_R = crate::FieldReader;
/// Field `STOP_BIT_NUM` writer - This register is used to set the length of stop bit.
pub type STOP_BIT_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 2>;

/// Field `RXFIFO_RST` writer - Set this bit to reset the uart receive-FIFO.
pub type RXFIFO_RST_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TXFIFO_RST` writer - Set this bit to reset the uart transmit-FIFO.
pub type TXFIFO_RST_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `ERR_WR_MASK` writer
/// 1: Receiver stops storing data into FIFO when data is wrong.
/// 0: Receiver stores the data even if the received data is wrong.
pub type ERR_WR_MASK_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bits 2:3 - This register is used to set the length of data.
    #[inline(always)]
    pub fn bit_num(&self) -> BIT_NUM_R {
        BIT_NUM_R::new(((self.bits >> 2) & 3) as u8)
    }

    /// Bit 1 - Set this bit to enable uart parity check.
    #[inline(always)]
    pub fn parity_en(&self) -> PARITY_EN_R {
        PARITY_EN_R::new(((self.bits >> 1) & 1) != 0)
    }

    /// Bits 4:5 - This register is used to set the length of stop bit.
    #[inline(always)]
    pub fn stop_bit_num(&self) -> STOP_BIT_NUM_R {
        STOP_BIT_NUM_R::new(((self.bits >> 4) & 3) as u8)
    }
}

impl W {
    /// Bit 0 - This register is used to configure the parity check mode.
    #[inline(always)]
    pub fn parity(&mut self) -> PARITY_W<CONF0_SPEC> {
        PARITY_W::new(self, 0)
    }

    /// Bit 1 - Set this bit to enable uart parity check.
    #[inline(always)]
    pub fn parity_en(&mut self) -> PARITY_EN_W<CONF0_SPEC> {
        PARITY_EN_W::new(self, 1)
    }

    /// Bits 2:3 - This register is used to set the length of data.
    #[inline(always)]
    pub fn bit_num(&mut self) -> BIT_NUM_W<CONF0_SPEC> {
        BIT_NUM_W::new(self, 2)
    }

    /// Bits 4:5 - This register is used to set the length of stop bit.
    #[inline(always)]
    pub fn stop_bit_num(&mut self) -> STOP_BIT_NUM_W<CONF0_SPEC> {
        STOP_BIT_NUM_W::new(self, 4)
    }

    /// Bit 17 - Set this bit to reset the uart receive-FIFO.
    #[inline(always)]
    pub fn rxfifo_rst(&mut self) -> RXFIFO_RST_W<CONF0_SPEC> {
        RXFIFO_RST_W::new(self, 17)
    }

    /// Bit 18 - Set this bit to reset the uart transmit-FIFO.
    #[inline(always)]
    pub fn txfifo_rst(&mut self) -> TXFIFO_RST_W<CONF0_SPEC> {
        TXFIFO_RST_W::new(self, 18)
    }

    /// Bit 26
    /// 1: Receiver stops storing data into FIFO when data is wrong.
    /// 0: Receiver stores the data even if the received data is wrong.
    #[inline(always)]
    pub fn err_wr_mask(&mut self) -> ERR_WR_MASK_W<CONF0_SPEC> {
        ERR_WR_MASK_W::new(self, 26)
    }
}

pub struct CONF0_SPEC;
impl crate::RegisterSpec for CONF0_SPEC {
    type Ux = u32;
}

impl crate::Readable for CONF0_SPEC {}

impl crate::Writable for CONF0_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
