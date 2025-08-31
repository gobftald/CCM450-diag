/// Register `STATUS` reader
pub type R = crate::R<STATUS_SPEC>;

/// Field `RXFIFO_CNT` reader - Stores the byte number of valid data in Rx-FIFO.
pub type RXFIFO_CNT_R = crate::FieldReader<u16>;
/// Field `TXFIFO_CNT` reader - Stores the byte number of data in Tx-FIFO.
pub type TXFIFO_CNT_R = crate::FieldReader<u16>;

impl R {
    /// Bits 0:9 - Stores the byte number of valid data in Rx-FIFO.
    #[inline(always)]
    pub fn rxfifo_cnt(&self) -> RXFIFO_CNT_R {
        RXFIFO_CNT_R::new((self.bits & 0x03ff) as u16)
    }

    /// Bits 16:25 - Stores the byte number of data in Tx-FIFO.
    #[inline(always)]
    pub fn txfifo_cnt(&self) -> TXFIFO_CNT_R {
        TXFIFO_CNT_R::new(((self.bits >> 16) & 0x03ff) as u16)
    }
}

pub struct STATUS_SPEC;
impl crate::RegisterSpec for STATUS_SPEC {
    type Ux = u32;
}

impl crate::Readable for STATUS_SPEC {}
