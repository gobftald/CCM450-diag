/// Register `AT_CMD_GAPTOUT` writer
pub type W = crate::W<AT_CMD_GAPTOUT_SPEC>;

/// Field `RX_GAP_TOUT` writer - This register is used to configure the duration time between the at_cmd chars.
pub type RX_GAP_TOUT_W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;

impl W {
    /// Bits 0:15 - This register is used to configure the duration time between the at_cmd chars.
    #[inline(always)]
    pub fn rx_gap_tout(&mut self) -> RX_GAP_TOUT_W<AT_CMD_GAPTOUT_SPEC> {
        RX_GAP_TOUT_W::new(self, 0)
    }
}

pub struct AT_CMD_GAPTOUT_SPEC;
impl crate::RegisterSpec for AT_CMD_GAPTOUT_SPEC {
    type Ux = u32;
}

impl crate::Writable for AT_CMD_GAPTOUT_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for AT_CMD_GAPTOUT_SPEC {
    const RESET_VALUE: u32 = 0x0b;
}
