/// Register `FSM_STATUS` reader
pub type R = crate::R<FSM_STATUS_SPEC>;

/// Field `ST_UTX_OUT` reader - This is the status register of transmitter.
pub type ST_UTX_OUT_R = crate::FieldReader;

impl R {
    /// "Bits 4:7 - This is the status register of transmitter.
    #[inline(always)]
    pub fn st_utx_out(&self) -> ST_UTX_OUT_R {
        ST_UTX_OUT_R::new(((self.bits >> 4) & 0x0f) as u8)
    }
}

pub struct FSM_STATUS_SPEC;
impl crate::RegisterSpec for FSM_STATUS_SPEC {
    type Ux = u32;
}

impl crate::Readable for FSM_STATUS_SPEC {}
