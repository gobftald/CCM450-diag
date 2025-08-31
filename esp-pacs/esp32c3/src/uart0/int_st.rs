/// Register `INT_ST` reader
pub type R = crate::R<INT_ST_SPEC>;

/// Field `RXFIFO_FULL` reader - This is the status bit for rxfifo_full_int_raw when rxfifo_full_int_ena is set to 1
pub type RXFIFO_FULL_R = crate::BitReader;

/// Field `TXFIFO_EMPTY` reader - This is the status bit for txfifo_empty_int_raw when txfifo_empty_int_ena is set to 1
pub type TXFIFO_EMPTY_R = crate::BitReader;

/// Field `PARITY_ERR` reader - This is the status bit for parity_err_int_raw when parity_err_int_ena is set to 1
pub type PARITY_ERR_R = crate::BitReader;

/// Field `FRM_ERR` reader - This is the status bit for frm_err_int_raw when frm_err_int_ena is set to 1
pub type FRM_ERR_R = crate::BitReader;

/// Field `RXFIFO_OVF` reader - This is the status bit for rxfifo_ovf_int_raw when rxfifo_ovf_int_ena is set to 1
pub type RXFIFO_OVF_R = crate::BitReader;

/// Field `RXFIFO_TOUT` reader - This is the status bit for rxfifo_tout_int_raw when rxfifo_tout_int_ena is set to 1
pub type RXFIFO_TOUT_R = crate::BitReader;

/// Field `GLITCH_DET` reader - This is the status bit for glitch_det_int_raw when glitch_det_int_ena is set to 1
pub type GLITCH_DET_R = crate::BitReader;

/// Field `TX_DONE` reader - This is the status bit for tx_done_int_raw when tx_done_int_ena is set to 1
pub type TX_DONE_R = crate::BitReader;

/// Field `AT_CMD_CHAR_DET` reader - This is the status bit for at_cmd_det_int_raw when at_cmd_char_det_int_ena is set to 1
pub type AT_CMD_CHAR_DET_R = crate::BitReader;

impl R {
    /// Bit 0 - This is the status bit for rxfifo_full_int_raw when rxfifo_full_int_ena is set to 1
    #[inline(always)]
    pub fn rxfifo_full(&self) -> RXFIFO_FULL_R {
        RXFIFO_FULL_R::new((self.bits & 1) != 0)
    }

    /// Bit 1 - This is the status bit for txfifo_empty_int_raw when txfifo_empty_int_ena is set to 1
    #[inline(always)]
    pub fn txfifo_empty(&self) -> TXFIFO_EMPTY_R {
        TXFIFO_EMPTY_R::new(((self.bits >> 1) & 1) != 0)
    }

    /// Bit 2 - This is the status bit for parity_err_int_raw when parity_err_int_ena is set to 1
    #[inline(always)]
    pub fn parity_err(&self) -> PARITY_ERR_R {
        PARITY_ERR_R::new(((self.bits >> 2) & 1) != 0)
    }

    /// Bit 3 - This is the status bit for frm_err_int_raw when frm_err_int_ena is set to 1.
    #[inline(always)]
    pub fn frm_err(&self) -> FRM_ERR_R {
        FRM_ERR_R::new(((self.bits >> 3) & 1) != 0)
    }

    /// Bit 4 - This is the status bit for rxfifo_ovf_int_raw when rxfifo_ovf_int_ena is set to 1
    #[inline(always)]
    pub fn rxfifo_ovf(&self) -> RXFIFO_OVF_R {
        RXFIFO_OVF_R::new(((self.bits >> 4) & 1) != 0)
    }

    /// Bit 8 - This is the status bit for rxfifo_tout_int_raw when rxfifo_tout_int_ena is set to 1
    #[inline(always)]
    pub fn rxfifo_tout(&self) -> RXFIFO_TOUT_R {
        RXFIFO_TOUT_R::new(((self.bits >> 8) & 1) != 0)
    }

    /// Bit 11 - This is the status bit for glitch_det_int_raw when glitch_det_int_ena is set to 1.
    #[inline(always)]
    pub fn glitch_det(&self) -> GLITCH_DET_R {
        GLITCH_DET_R::new(((self.bits >> 11) & 1) != 0)
    }

    /// Bit 14 - This is the status bit for tx_done_int_raw when tx_done_int_ena is set to 1
    #[inline(always)]
    pub fn tx_done(&self) -> TX_DONE_R {
        TX_DONE_R::new(((self.bits >> 14) & 1) != 0)
    }

    /// Bit 18 - This is the status bit for at_cmd_det_int_raw when at_cmd_char_det_int_ena is set to 1."]
    #[inline(always)]
    pub fn at_cmd_char_det(&self) -> AT_CMD_CHAR_DET_R {
        AT_CMD_CHAR_DET_R::new(((self.bits >> 18) & 1) != 0)
    }
}

pub struct INT_ST_SPEC;
impl crate::RegisterSpec for INT_ST_SPEC {
    type Ux = u32;
}

impl crate::Readable for INT_ST_SPEC {}
