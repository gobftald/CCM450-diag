/// Register `IDLE_CONF` writer
pub type W = crate::W<IDLE_CONF_SPEC>;

/// Field `TX_IDLE_NUM` writer - This register is used to configure the duration time between transfers.
pub type TX_IDLE_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;

impl W {
    /// Bits 10:19 - This register is used to configure the duration time between transfers.
    #[inline(always)]
    pub fn tx_idle_num(&mut self) -> TX_IDLE_NUM_W<IDLE_CONF_SPEC> {
        TX_IDLE_NUM_W::new(self, 10)
    }
}

pub struct IDLE_CONF_SPEC;
impl crate::RegisterSpec for IDLE_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for IDLE_CONF_SPEC {}

impl crate::Writable for IDLE_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
