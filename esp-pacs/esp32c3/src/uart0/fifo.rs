/// Register `FIFO` reader
pub type R = crate::R<FIFO_SPEC>;
/// "Register `FIFO` writer
pub type W = crate::W<FIFO_SPEC>;

/// Field `RXFIFO_RD_BYTE` reader - UART 0 accesses FIFO via this register.
pub type RXFIFO_RD_BYTE_R = crate::FieldReader;
/// Field `RXFIFO_RD_BYTE` writer - UART 0 accesses FIFO via this register.
pub type RXFIFO_RD_BYTE_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

impl R {
    /// Bits 0:7 - UART 0 accesses FIFO via this register.
    #[inline(always)]
    pub fn rxfifo_rd_byte(&self) -> RXFIFO_RD_BYTE_R {
        RXFIFO_RD_BYTE_R::new((self.bits & 0xff) as u8)
    }
}

impl W {
    /// Bits 0:7 - UART 0 accesses FIFO via this register.
    #[inline(always)]
    pub fn rxfifo_rd_byte(&mut self) -> RXFIFO_RD_BYTE_W<FIFO_SPEC> {
        RXFIFO_RD_BYTE_W::new(self, 0)
    }
}

pub struct FIFO_SPEC;
impl crate::RegisterSpec for FIFO_SPEC {
    type Ux = u32;
}

impl crate::Readable for FIFO_SPEC {}

impl crate::Writable for FIFO_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for FIFO_SPEC {
    const RESET_VALUE: u32 = 0;
}
