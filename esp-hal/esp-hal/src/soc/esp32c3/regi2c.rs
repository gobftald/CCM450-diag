use crate::rom::regi2c::{define_regi2c, RawRegI2cField, RegI2cMaster, RegI2cRegister};

define_regi2c! {
    master: REGI2C_BBPLL(0x66, 0) {
        // 16
        reg: I2C_BBPLL_OC_REF(2) {
            field: I2C_BBPLL_OC_ENB_FCAL(7..7),
            field: I2C_BBPLL_OC_DCHGP(6..4),
            field: I2C_BBPLL_OC_REF_DIV(3..0)
        }

        // 21
        reg: I2C_BBPLL_OC_DIV_REG(3) {
            field: I2C_BBPLL_OC_DIV(7..0)
        }

        // 24
        reg: I2C_BBPLL_REG4(4) {
            field: I2C_BBPLL_OC_TSCHGP(7..7),
            field: I2C_BBPLL_OC_ENB_VCON(6..6),
            field: I2C_BBPLL_DIV_CPU(5..5),
            field: I2C_BBPLL_DIV_DAC(4..4),
            field: I2C_BBPLL_DIV_ADC(3..2),
            field: I2C_BBPLL_MODE_HF(1..1),
            field: I2C_BBPLL_RSTB_DIV_ADC(0..0)
        }

        // 33
        reg: I2C_BBPLL_OC_DR(5) {
            field: I2C_BBPLL_EN_USB(7..7),
            field: I2C_BBPLL_OC_DR3(6..4),
            field: I2C_BBPLL_OC_DR1(2..0)
        }

        // 38
        reg: I2C_BBPLL_REG6(6) {
            field: I2C_BBPLL_OC_DLREF_SEL(7..6),
            field: I2C_BBPLL_OC_DHREF_SEL(5..4),
            field: I2C_BBPLL_INC_CUR(3..3),
            field: I2C_BBPLL_OC_DCUR(2..0)
        }

        // 51
        reg: I2C_BBPLL_REG9(9) {
            field: I2C_BBPLL_BBADC_DREF(7..6),
            field: I2C_BBPLL_BBADC_DVDD(5..4),
            field: I2C_BBPLL_BBADC_DELAY2(3..2),
            field: I2C_BBPLL_OC_VCO_DBIAS(1..0)
        }
    }
}

// 146
pub(crate) fn regi2c_read(block: u8, host_id: u8, reg_add: u8) -> u8 {
    unsafe extern "C" {
        pub(crate) fn esp_rom_regi2c_read(block: u8, block_hostid: u8, reg_add: u8) -> u8;
    }
    unsafe { esp_rom_regi2c_read(block, host_id, reg_add) }
}

// 153
pub(crate) fn regi2c_write(block: u8, host_id: u8, reg_add: u8, data: u8) {
    unsafe extern "C" {
        pub(crate) fn rom_i2c_writeReg(block: u8, block_hostid: u8, reg_add: u8, indata: u8);
    }
    unsafe { rom_i2c_writeReg(block, host_id, reg_add, data) };
}
