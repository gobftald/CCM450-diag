/// Register `AT_CMD_PRECNT` writer
pub type W = crate::W<AT_CMD_PRECNT_SPEC>;

/// Field `PRE_IDLE_NUM` writer - This register is used to configure
/// the idle duration time before the first at_cmd is received by receiver.
pub type PRE_IDLE_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;

impl W {
    /// Bits 0:15 - This register is used to configure the idle duration time
    /// before the first at_cmd is received by receiver.
    #[inline(always)]
    pub fn pre_idle_num(&mut self) -> PRE_IDLE_NUM_W<AT_CMD_PRECNT_SPEC> {
        PRE_IDLE_NUM_W::new(self, 0)
    }
}

pub struct AT_CMD_PRECNT_SPEC;
impl crate::RegisterSpec for AT_CMD_PRECNT_SPEC {
    type Ux = u32;
}

impl crate::Writable for AT_CMD_PRECNT_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for AT_CMD_PRECNT_SPEC {
    const RESET_VALUE: u32 = 0x0901;
}
