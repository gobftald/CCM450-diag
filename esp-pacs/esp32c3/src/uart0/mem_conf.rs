/// Register `MEM_CONF` writer"]
pub type W = crate::W<MEM_CONF_SPEC>;

/// Field `RX_TOUT_THRHD` writer - This register is used to configure the threshold time
/// that receiver takes to receive one byte. The rxfifo_tout_int interrupt will be trigger
/// when the receiver takes more time to receive one byte with rx_tout_en set to 1.
pub type RX_TOUT_THRHD_W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;

impl W {
    /// Bits 16:25 - This register is used to configure the threshold time
    /// that receiver takes to receive one byte. The rxfifo_tout_int interrupt will be trigger
    /// when the receiver takes more time to receive one byte with rx_tout_en set to 1.
    #[inline(always)]
    pub fn rx_tout_thrhd(&mut self) -> RX_TOUT_THRHD_W<MEM_CONF_SPEC> {
        RX_TOUT_THRHD_W::new(self, 16)
    }
}

pub struct MEM_CONF_SPEC;
impl crate::RegisterSpec for MEM_CONF_SPEC {
    type Ux = u32;
}

impl crate::Readable for MEM_CONF_SPEC {}

impl crate::Writable for MEM_CONF_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
