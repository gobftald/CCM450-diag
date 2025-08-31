/// Register `AT_CMD_CHAR` reader
pub type R = crate::R<AT_CMD_CHAR_SPEC>;
/// Register `AT_CMD_CHAR` writer
pub type W = crate::W<AT_CMD_CHAR_SPEC>;

/// Field `AT_CMD_CHAR` writer - This register is used to configure
/// the content of at_cmd char.
pub type AT_CMD_CHAR_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

/// Field `CHAR_NUM` reader - This register is used to configure
/// the num of continuous at_cmd chars received by receiver.
pub type CHAR_NUM_R = crate::FieldReader;
/// Field `CHAR_NUM` writer - This register is used to configure
/// the num of continuous at_cmd chars received by receiver.
pub type CHAR_NUM_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

impl R {
    /// Bits 8:15 - This register is used to configure the num of continuous
    /// at_cmd chars received by receiver.
    #[inline(always)]
    pub fn char_num(&self) -> CHAR_NUM_R {
        CHAR_NUM_R::new(((self.bits >> 8) & 0xff) as u8)
    }
}

impl W {
    /// Bits 0:7 - This register is used to configure the content of at_cmd char.
    #[inline(always)]
    pub fn at_cmd_char(&mut self) -> AT_CMD_CHAR_W<AT_CMD_CHAR_SPEC> {
        AT_CMD_CHAR_W::new(self, 0)
    }

    /// Bits 8:15 - This register is used to configure the num of continuous at_cmd chars received by receiver.
    #[inline(always)]
    pub fn char_num(&mut self) -> CHAR_NUM_W<AT_CMD_CHAR_SPEC> {
        CHAR_NUM_W::new(self, 8)
    }
}

pub struct AT_CMD_CHAR_SPEC;
impl crate::RegisterSpec for AT_CMD_CHAR_SPEC {
    type Ux = u32;
}

impl crate::Readable for AT_CMD_CHAR_SPEC {}

impl crate::Writable for AT_CMD_CHAR_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for AT_CMD_CHAR_SPEC {
    const RESET_VALUE: u32 = 0x032b;
}
