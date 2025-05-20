/// Register `OPTIONS0` writer
pub type W = crate::W<OPTIONS0_SPEC>;

/// Field `BB_I2C_FORCE_PD` writer - BB_I2C force power down
pub type BB_I2C_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BB_I2C_FORCE_PU` writer - BB_I2C force power up
pub type BB_I2C_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BBPLL_I2C_FORCE_PD` writer - BB_PLL _I2C force power down
pub type BBPLL_I2C_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BBPLL_I2C_FORCE_PU` writer - BB_PLL_I2C force power up
pub type BBPLL_I2C_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BBPLL_FORCE_PD` writer - BB_PLL force power down
pub type BBPLL_FORCE_PD_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `BBPLL_FORCE_PU` writer - BB_PLL force power up
pub type BBPLL_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

/// Field `XTL_FORCE_PU` writer - crystall force power up
pub type XTL_FORCE_PU_W<'a, REG> = crate::BitWriter<'a, REG>;

impl W {
    /// Bit 6 - BB_I2C force power down
    #[inline(always)]
    pub fn bb_i2c_force_pd(&mut self) -> BB_I2C_FORCE_PD_W<OPTIONS0_SPEC> {
        BB_I2C_FORCE_PD_W::new(self, 6)
    }

    /// Bit 7 - BB_I2C force power up
    #[inline(always)]
    pub fn bb_i2c_force_pu(&mut self) -> BB_I2C_FORCE_PU_W<OPTIONS0_SPEC> {
        BB_I2C_FORCE_PU_W::new(self, 7)
    }

    /// Bit 8 - BB_PLL _I2C force power down
    #[inline(always)]
    pub fn bbpll_i2c_force_pd(&mut self) -> BBPLL_I2C_FORCE_PD_W<OPTIONS0_SPEC> {
        BBPLL_I2C_FORCE_PD_W::new(self, 8)
    }

    /// Bit 9 - BB_PLL_I2C force power up
    #[inline(always)]
    pub fn bbpll_i2c_force_pu(&mut self) -> BBPLL_I2C_FORCE_PU_W<OPTIONS0_SPEC> {
        BBPLL_I2C_FORCE_PU_W::new(self, 9)
    }

    /// Bit 10 - BB_PLL force power down
    #[inline(always)]
    pub fn bbpll_force_pd(&mut self) -> BBPLL_FORCE_PD_W<OPTIONS0_SPEC> {
        BBPLL_FORCE_PD_W::new(self, 10)
    }

    /// Bit 11 - BB_PLL force power up
    #[inline(always)]
    pub fn bbpll_force_pu(&mut self) -> BBPLL_FORCE_PU_W<OPTIONS0_SPEC> {
        BBPLL_FORCE_PU_W::new(self, 11)
    }

    /// Bit 13 - crystall force power up
    #[inline(always)]
    pub fn xtl_force_pu(&mut self) -> XTL_FORCE_PU_W<OPTIONS0_SPEC> {
        XTL_FORCE_PU_W::new(self, 13)
    }
}

pub struct OPTIONS0_SPEC;
impl crate::RegisterSpec for OPTIONS0_SPEC {
    type Ux = u32;
}

impl crate::Readable for OPTIONS0_SPEC {}

impl crate::Writable for OPTIONS0_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
