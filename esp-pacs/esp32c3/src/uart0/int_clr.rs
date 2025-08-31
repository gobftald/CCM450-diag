/// Register `INT_CLR` writer
pub type W = crate::W<INT_CLR_SPEC>;

/// Field `RXFIFO_FULL` writer - Set this bit to clear the rxfifo_full_int_raw interrupt.
pub type RXFIFO_FULL_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `TXFIFO_EMPTY` writer - Set this bit to clear txfifo_empty_int_raw interrupt.
pub type TXFIFO_EMPTY_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `RXFIFO_OVF` writer - Set this bit to clear rxfifo_ovf_int_raw interrupt.
pub type RXFIFO_OVF_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `RXFIFO_TOUT` writer - Set this bit to clear the rxfifo_tout_int_raw interrupt.
pub type RXFIFO_TOUT_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `TX_BRK_DONE` writer - Set this bit to clear the tx_brk_done_int_raw interrupt..
pub type TX_BRK_DONE_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `TX_BRK_IDLE_DONE` writer - Set this bit to clear the tx_brk_idle_done_int_raw interrupt.
pub type TX_BRK_IDLE_DONE_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `TX_DONE` writer - Set this bit to clear the tx_done_int_raw interrupt.
pub type TX_DONE_W<'a, REG> = crate::BitWriter1C<'a, REG>;

/// Field `AT_CMD_CHAR_DET` writer - Set this bit to clear the at_cmd_char_det_int_raw interrupt.
pub type AT_CMD_CHAR_DET_W<'a, REG> = crate::BitWriter1C<'a, REG>;

impl W {
    /// Bit 0 - Set this bit to clear the rxfifo_full_int_raw interrupt.
    #[inline(always)]
    pub fn rxfifo_full(&mut self) -> RXFIFO_FULL_W<INT_CLR_SPEC> {
        RXFIFO_FULL_W::new(self, 0)
    }

    /// Bit 1 - Set this bit to clear txfifo_empty_int_raw interrupt.
    #[inline(always)]
    pub fn txfifo_empty(&mut self) -> TXFIFO_EMPTY_W<INT_CLR_SPEC> {
        TXFIFO_EMPTY_W::new(self, 1)
    }

    /// Bit 4 - Set this bit to clear rxfifo_ovf_int_raw interrupt.
    #[inline(always)]
    pub fn rxfifo_ovf(&mut self) -> RXFIFO_OVF_W<INT_CLR_SPEC> {
        RXFIFO_OVF_W::new(self, 4)
    }

    /// Bit 8 - Set this bit to clear the rxfifo_tout_int_raw interrupt.
    #[inline(always)]
    pub fn rxfifo_tout(&mut self) -> RXFIFO_TOUT_W<INT_CLR_SPEC> {
        RXFIFO_TOUT_W::new(self, 8)
    }

    /// Bit 12 - Set this bit to clear the tx_brk_done_int_raw interrupt..
    #[inline(always)]
    pub fn tx_brk_done(&mut self) -> TX_BRK_DONE_W<INT_CLR_SPEC> {
        TX_BRK_DONE_W::new(self, 12)
    }

    /// Bit 13 - Set this bit to clear the tx_brk_idle_done_int_raw interrupt.
    #[inline(always)]
    pub fn tx_brk_idle_done(&mut self) -> TX_BRK_IDLE_DONE_W<INT_CLR_SPEC> {
        TX_BRK_IDLE_DONE_W::new(self, 13)
    }

    /// Bit 14 - Set this bit to clear the tx_done_int_raw interrupt.
    #[inline(always)]
    pub fn tx_done(&mut self) -> TX_DONE_W<INT_CLR_SPEC> {
        TX_DONE_W::new(self, 14)
    }

    /// Bit 18 - Set this bit to clear the at_cmd_char_det_int_raw interrupt.
    #[inline(always)]
    pub fn at_cmd_char_det(&mut self) -> AT_CMD_CHAR_DET_W<INT_CLR_SPEC> {
        AT_CMD_CHAR_DET_W::new(self, 18)
    }
}

pub struct INT_CLR_SPEC;
impl crate::RegisterSpec for INT_CLR_SPEC {
    type Ux = u32;
}

impl crate::Writable for INT_CLR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x000f_ffff;
}

impl crate::Resettable for INT_CLR_SPEC {
    const RESET_VALUE: u32 = 0;
}
