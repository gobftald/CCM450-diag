/// Register `INT_ENA` writer
pub type W = crate::W<INT_ENA_SPEC>;

/// Field `RXFIFO_FULL` writer - This is the enable bit for rxfifo_full_int_st register.
pub type RXFIFO_FULL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TXFIFO_EMPTY` writer - This is the enable bit for txfifo_empty_int_st register.
pub type TXFIFO_EMPTY_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `PARITY_ERR` writer - This is the enable bit for parity_err_int_st register.
pub type PARITY_ERR_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `FRM_ERR` writer - This is the enable bit for frm_err_int_st register.
pub type FRM_ERR_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `RXFIFO_OVF` writer - This is the enable bit for rxfifo_ovf_int_st register.
pub type RXFIFO_OVF_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `RXFIFO_TOUT` writer - This is the enable bit for rxfifo_tout_int_st register.
pub type RXFIFO_TOUT_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `GLITCH_DET` writer - This is the enable bit for glitch_det_int_st register.
pub type GLITCH_DET_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TX_BRK_DONE` writer - This is the enable bit for tx_brk_done_int_st register.
pub type TX_BRK_DONE_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TX_BRK_IDLE_DONE` writer - This is the enable bit for tx_brk_idle_done_int_st register.
pub type TX_BRK_IDLE_DONE_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `TX_DONE` writer - This is the enable bit for tx_done_int_st register.
pub type TX_DONE_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `AT_CMD_CHAR_DET` writer - This is the enable bit for at_cmd_char_det_int_st register.
pub type AT_CMD_CHAR_DET_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 0 - This is the enable bit for rxfifo_full_int_st register.
    #[inline(always)]
    pub fn rxfifo_full(&mut self) -> RXFIFO_FULL_W<INT_ENA_SPEC> {
        RXFIFO_FULL_W::new(self, 0)
    }

    /// Bit 1 - This is the enable bit for txfifo_empty_int_st register.
    #[inline(always)]
    pub fn txfifo_empty(&mut self) -> TXFIFO_EMPTY_W<INT_ENA_SPEC> {
        TXFIFO_EMPTY_W::new(self, 1)
    }

    /// Bit 2 - This is the enable bit for parity_err_int_st register.
    #[inline(always)]
    pub fn parity_err(&mut self) -> PARITY_ERR_W<INT_ENA_SPEC> {
        PARITY_ERR_W::new(self, 2)
    }

    /// Bit 3 - This is the enable bit for frm_err_int_st register.
    #[inline(always)]
    pub fn frm_err(&mut self) -> FRM_ERR_W<INT_ENA_SPEC> {
        FRM_ERR_W::new(self, 3)
    }

    /// Bit 4 - This is the enable bit for rxfifo_ovf_int_st register.
    #[inline(always)]
    pub fn rxfifo_ovf(&mut self) -> RXFIFO_OVF_W<INT_ENA_SPEC> {
        RXFIFO_OVF_W::new(self, 4)
    }

    /// Bit 8 - This is the enable bit for rxfifo_tout_int_st register.
    #[inline(always)]
    pub fn rxfifo_tout(&mut self) -> RXFIFO_TOUT_W<INT_ENA_SPEC> {
        RXFIFO_TOUT_W::new(self, 8)
    }

    /// Bit 11 - This is the enable bit for glitch_det_int_st register.
    #[inline(always)]
    pub fn glitch_det(&mut self) -> GLITCH_DET_W<INT_ENA_SPEC> {
        GLITCH_DET_W::new(self, 11)
    }

    /// Bit 12 - This is the enable bit for tx_brk_done_int_st register.
    #[inline(always)]
    pub fn tx_brk_done(&mut self) -> TX_BRK_DONE_W<INT_ENA_SPEC> {
        TX_BRK_DONE_W::new(self, 12)
    }

    /// Bit 13 - This is the enable bit for tx_brk_idle_done_int_st register.
    #[inline(always)]
    pub fn tx_brk_idle_done(&mut self) -> TX_BRK_IDLE_DONE_W<INT_ENA_SPEC> {
        TX_BRK_IDLE_DONE_W::new(self, 13)
    }

    /// Bit 14 - This is the enable bit for tx_done_int_st register.
    #[inline(always)]
    pub fn tx_done(&mut self) -> TX_DONE_W<INT_ENA_SPEC> {
        TX_DONE_W::new(self, 14)
    }

    /// Bit 18 - This is the enable bit for at_cmd_char_det_int_st register.
    #[inline(always)]
    pub fn at_cmd_char_det(&mut self) -> AT_CMD_CHAR_DET_W<INT_ENA_SPEC> {
        AT_CMD_CHAR_DET_W::new(self, 18)
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

impl crate::Resettable for INT_ENA_SPEC {
    const RESET_VALUE: u32 = 0;
}
