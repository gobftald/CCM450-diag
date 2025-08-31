/// Register `INT_RAW` reader
pub type R = crate::R<INT_RAW_SPEC>;

/// Field `RXFIFO_FULL` reader - This interrupt raw bit turns to high level when receiver
/// receives more data than what rxfifo_full_thrhd specifies.
pub type RXFIFO_FULL_R = crate::BitReader;

/// Field `TXFIFO_EMPTY` reader - This interrupt raw bit turns to high level when
/// the amount of data in Tx-FIFO is less than what txfifo_empty_thrhd specifies
pub type TXFIFO_EMPTY_R = crate::BitReader;
/// Field `PARITY_ERR` reader - This interrupt raw bit turns to high level when receiver detects a parity error in the data.
pub type PARITY_ERR_R = crate::BitReader;

/// Field `FRM_ERR` reader - This interrupt raw bit turns to high level when receiver detects a data frame error .
pub type FRM_ERR_R = crate::BitReader;

/// Field `RXFIFO_OVF` reader - This interrupt raw bit turns to high level
/// when receiver receives more data than the FIFO can store.
pub type RXFIFO_OVF_R = crate::BitReader;

/// Field `RXFIFO_TOUT` reader - This interrupt raw bit turns to high level
/// when receiver takes more time than rx_tout_thrhd to receive a byte.
pub type RXFIFO_TOUT_R = crate::BitReader;

/// Field `GLITCH_DET` reader - This interrupt raw bit turns to high level
/// when receiver detects a glitch in the middle of a start bit.
pub type GLITCH_DET_R = crate::BitReader;

/// Field `TX_DONE` reader - This interrupt raw bit turns to high level when transmitter has send out all data in FIFO.
pub type TX_DONE_R = crate::BitReader;

/// Field `AT_CMD_CHAR_DET` reader - This interrupt raw bit turns to high level
/// when receiver detects the configured at_cmd char.
pub type AT_CMD_CHAR_DET_R = crate::BitReader;

impl R {
    /// Bit 0 - This interrupt raw bit turns to high level when receiver
    /// receives more data than what rxfifo_full_thrhd specifies.
    #[inline(always)]
    pub fn rxfifo_full(&self) -> RXFIFO_FULL_R {
        RXFIFO_FULL_R::new((self.bits & 1) != 0)
    }

    /// Bit 1 - This interrupt raw bit turns to high level when the amount of data
    /// in Tx-FIFOis less than what txfifo_empty_thrhd specifies .
    #[inline(always)]
    pub fn txfifo_empty(&self) -> TXFIFO_EMPTY_R {
        TXFIFO_EMPTY_R::new(((self.bits >> 1) & 1) != 0)
    }

    /// Bit 2 - This interrupt raw bit turns to high level when receiver detects a parity error in the data.
    #[inline(always)]
    pub fn parity_err(&self) -> PARITY_ERR_R {
        PARITY_ERR_R::new(((self.bits >> 2) & 1) != 0)
    }

    /// Bit 3 - This interrupt raw bit turns to high level when receiver detects a data frame error .
    #[inline(always)]
    pub fn frm_err(&self) -> FRM_ERR_R {
        FRM_ERR_R::new(((self.bits >> 3) & 1) != 0)
    }

    /// Bit 4 - This interrupt raw bit turns to high level when receiver receives more data than the FIFO can store.
    #[inline(always)]
    pub fn rxfifo_ovf(&self) -> RXFIFO_OVF_R {
        RXFIFO_OVF_R::new(((self.bits >> 4) & 1) != 0)
    }

    /// Bit 8 - This interrupt raw bit turns to high level when receiver
    /// takes more time than rx_tout_thrhd to receive a byte.
    #[inline(always)]
    pub fn rxfifo_tout(&self) -> RXFIFO_TOUT_R {
        RXFIFO_TOUT_R::new(((self.bits >> 8) & 1) != 0)
    }

    /// Bit 11 - This interrupt raw bit turns to high level when receiver detects a glitch in the middle of a start bit.
    #[inline(always)]
    pub fn glitch_det(&self) -> GLITCH_DET_R {
        GLITCH_DET_R::new(((self.bits >> 11) & 1) != 0)
    }

    /// Bit 14 - This interrupt raw bit turns to high level when transmitter has send out all data in FIFO.
    #[inline(always)]
    pub fn tx_done(&self) -> TX_DONE_R {
        TX_DONE_R::new(((self.bits >> 14) & 1) != 0)
    }

    /// Bit 18 - This interrupt raw bit turns to high level
    /// when receiver detects the configured at_cmd char.
    #[inline(always)]
    pub fn at_cmd_char_det(&self) -> AT_CMD_CHAR_DET_R {
        AT_CMD_CHAR_DET_R::new(((self.bits >> 18) & 1) != 0)
    }
}

pub struct INT_RAW_SPEC;
impl crate::RegisterSpec for INT_RAW_SPEC {
    type Ux = u32;
}

impl crate::Readable for INT_RAW_SPEC {}
