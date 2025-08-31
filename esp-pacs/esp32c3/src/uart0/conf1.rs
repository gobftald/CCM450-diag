/// Register `CONF1` reader
pub type R = crate::R<CONF1_SPEC>;
/// Register `CONF1` writer
pub type W = crate::W<CONF1_SPEC>;

/// Field `RXFIFO_FULL_THRHD` reader - It will produce rxfifo_full_int interrupt
/// when receiver receives more data than this register value.
pub type RXFIFO_FULL_THRHD_R = crate::FieldReader<u16>;
/// Field `RXFIFO_FULL_THRHD` writer - It will produce rxfifo_full_int interrupt
/// when receiver receives more data than this register value."]
pub type RXFIFO_FULL_THRHD_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `TXFIFO_EMPTY_THRHD` writer - It will produce txfifo_empty_int interrupt
/// when the data amount in Tx-FIFO is less than this register value.
pub type TXFIFO_EMPTY_THRHD_W<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;

/// Field `RX_TOUT_EN` reader - This is the enble bit for uart receiver's timeout function.
pub type RX_TOUT_EN_R = crate::BitReader;
/// Field `RX_TOUT_EN` writer - This is the enble bit for uart receiver's timeout function.
pub type RX_TOUT_EN_W<'a, REG> = crate::BitWriter<'a, REG>;

impl R {
    /// Bits 0:8 - It will produce rxfifo_full_int interrupt
    /// when receiver receives more data than this register value.
    #[inline(always)]
    pub fn rxfifo_full_thrhd(&self) -> RXFIFO_FULL_THRHD_R {
        RXFIFO_FULL_THRHD_R::new((self.bits & 0x01ff) as u16)
    }

    /// Bit 21 - This is the enble bit for uart receiver's timeout function.
    #[inline(always)]
    pub fn rx_tout_en(&self) -> RX_TOUT_EN_R {
        RX_TOUT_EN_R::new(((self.bits >> 21) & 1) != 0)
    }
}

impl W {
    /// Bits 0:8 - It will produce rxfifo_full_int interrupt
    /// when receiver receives more data than this register value.
    #[inline(always)]
    pub fn rxfifo_full_thrhd(&mut self) -> RXFIFO_FULL_THRHD_W<CONF1_SPEC> {
        RXFIFO_FULL_THRHD_W::new(self, 0)
    }

    /// Bits 9:17 - It will produce txfifo_empty_int interrupt
    /// when the data amount in Tx-FIFO is less than this register value.
    #[inline(always)]
    pub fn txfifo_empty_thrhd(&mut self) -> TXFIFO_EMPTY_THRHD_W<CONF1_SPEC> {
        TXFIFO_EMPTY_THRHD_W::new(self, 9)
    }

    /// Bit 21 - This is the enble bit for uart receiver's timeout function.
    #[inline(always)]
    pub fn rx_tout_en(&mut self) -> RX_TOUT_EN_W<CONF1_SPEC> {
        RX_TOUT_EN_W::new(self, 21)
    }
}

pub struct CONF1_SPEC;
impl crate::RegisterSpec for CONF1_SPEC {
    type Ux = u32;
}

impl crate::Readable for CONF1_SPEC {}

impl crate::Writable for CONF1_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
