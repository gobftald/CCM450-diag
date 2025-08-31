/// Register `AT_CMD_POSTCNT` writer
pub type W = crate::W<AT_CMD_POSTCNT_SPEC>;

/// Field `POST_IDLE_NUM` writer - This register is used to configure the duration time
/// between the last at_cmd and the next data.
pub type POST_IDLE_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;

impl W {
    /// Bits 0:15 - This register is used to configure the duration time between the last at_cmd and the next data.
    #[inline(always)]
    pub fn post_idle_num(&mut self) -> POST_IDLE_NUM_W<AT_CMD_POSTCNT_SPEC> {
        POST_IDLE_NUM_W::new(self, 0)
    }
}

pub struct AT_CMD_POSTCNT_SPEC;
impl crate::RegisterSpec for AT_CMD_POSTCNT_SPEC {
    type Ux = u32;
}

/// read()` method returns [`at_cmd_postcnt::R`](R) reader structure
impl crate::Readable for AT_CMD_POSTCNT_SPEC {}
#[doc = "`write(|w| ..)` method takes [`at_cmd_postcnt::W`](W) writer structure"]
impl crate::Writable for AT_CMD_POSTCNT_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for AT_CMD_POSTCNT_SPEC {
    const RESET_VALUE: u32 = 0x0901;
}
