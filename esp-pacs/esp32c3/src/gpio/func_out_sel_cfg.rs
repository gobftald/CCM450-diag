/// Register `FUNC%s_OUT_SEL_CFG` writer
pub type W = crate::W<FUNC_OUT_SEL_CFG_SPEC>;

/// Field `OUT_SEL` writer - The value of the bits: 0<=s<=256. Set the value to select output signal.
/// s=0-255: output of GPIO[n] equals input of peripheral[s].
/// s=256: output of GPIO[n] equals GPIO_OUT_REG[n]."]
pub type OUT_SEL_W<'a, REG> = crate::FieldWriter<'a, REG, 8>;

/// Field `INV_SEL` writer - set this bit to invert output signal.1:invert.0:not invert.
pub type INV_SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `OEN_SEL` writer - set this bit to select output enable signal.
/// 1:use GPIO_ENABLE_REG[n] as output enable signal.
/// 0:use peripheral output enable signal.
pub type OEN_SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `OEN_INV_SEL` writer - set this bit to invert output enable signal.1:invert.0:not invert.
pub type OEN_INV_SEL_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bits 0:7 - The value of the bits: 0<=s<=256. Set the value to select output signal.
    /// s=0-255: output of GPIO[n] equals input of peripheral[s].
    /// s=256: output of GPIO[n] equals GPIO_OUT_REG[n].
    #[inline(always)]
    pub fn out_sel(&mut self) -> OUT_SEL_W<FUNC_OUT_SEL_CFG_SPEC> {
        OUT_SEL_W::new(self, 0)
    }

    /// Bit 8 - set this bit to invert output signal.1:invert.0:not invert.
    #[inline(always)]
    pub fn inv_sel(&mut self) -> INV_SEL_W<FUNC_OUT_SEL_CFG_SPEC> {
        INV_SEL_W::new(self, 8)
    }

    /// Bit 9 - set this bit to select output enable signal.
    /// 1:use GPIO_ENABLE_REG[n] as output enable signal.
    /// 0:use peripheral output enable signal.
    #[inline(always)]
    pub fn oen_sel(&mut self) -> OEN_SEL_W<FUNC_OUT_SEL_CFG_SPEC> {
        OEN_SEL_W::new(self, 9)
    }

    /// Bit 10 - set this bit to invert output enable signal.1:invert.0:not invert.
    #[inline(always)]
    pub fn oen_inv_sel(&mut self) -> OEN_INV_SEL_W<FUNC_OUT_SEL_CFG_SPEC> {
        OEN_INV_SEL_W::new(self, 10)
    }
}

pub struct FUNC_OUT_SEL_CFG_SPEC;
impl crate::RegisterSpec for FUNC_OUT_SEL_CFG_SPEC {
    type Ux = u32;
}

impl crate::Readable for FUNC_OUT_SEL_CFG_SPEC {}

impl crate::Writable for FUNC_OUT_SEL_CFG_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}

impl crate::Resettable for FUNC_OUT_SEL_CFG_SPEC {
    const RESET_VALUE: u32 = 0x80;
}
