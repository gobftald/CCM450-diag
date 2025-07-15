use strum::FromRepr;

use crate::peripherals::{LPWR, SYSTEM};

// 10
pub(crate) fn init() {
    let rtc_cntl = LPWR::regs();

    /* we don't want to init RTC sleep modes, power down and power up params and method
     *
    // digital system voltage regulator is off when chip enters Light-sleep and Deep-sleep modes
    regi2c::I2C_DIG_REG_XPD_DIG_REG.write_field(0);
    // low power voltage regulator is off when chip enters Light-sleep and Deep-sleep modes
    regi2c::I2C_DIG_REG_XPD_RTC_REG.write_field(0);

    rtc_cntl
        // register of power options for I2C and PLLA
        .ana_conf()
        // pvtmon_pu bit is marked as 'reserved' in documenation
        .modify(|_, w| w.pvtmon_pu().clear_bit());

    unsafe {
        rtc_cntl
            // Configures CPU stall options
            .timer1()
            .modify(|_, w| {
                w
                    // PLL wait cycles in slow_clk_rtc
                    .pll_buf_wait()
                    .bits(20u8)
                    // CK8M wait cycles in slow_clk_rtc
                    // RTC_CNTL_FOSC_WAIT - Sets the FOSC
                    // (Fast RC Oscillator) clock waiting cycles (using the RTC slow clock)
                    .ck8m_wait()
                    .bits(20u8)
            });

        rtc_cntl
            // minimal sleep cycles in slow_clk_rtc
            .timer5()
            // Sets the minimal sleep cycles (using the RTC slow clock).
            .modify(|_, w| w.min_slp_val().bits(2u8));

        // Set default powerup & wait time
        rtc_cntl.timer3().modify(|_, w| {
            // wifi power domain power on time
            w.wifi_powerup_timer().bits(1u8);
            // wifi power domain wakeup time
            w.wifi_wait_timer().bits(1u16);
            // bt power domain power on time
            w.bt_powerup_timer().bits(1u8);
            // bt power domain wakeup time
            w.bt_wait_timer().bits(1u16)
        });
        rtc_cntl.timer4().modify(|_, w| {
            // cpu top power domain power on time
            w.cpu_top_powerup_timer().bits(1u8);
            // cpu top power domain wakeup time
            w.cpu_top_wait_timer().bits(1u16);
            // digital wrap power domain power on time
            w.dg_wrap_powerup_timer().bits(1u8);
            // digital wrap power domain wakeup time
            w.dg_wrap_wait_timer().bits(1u16)
        });
        rtc_cntl.timer6().modify(|_, w| {
            // digital peri power domain power on time
            w.dg_peri_powerup_timer().bits(1u8);
            // digital peri power domain wakeup time
            w.dg_peri_wait_timer().bits(1u16)
        });
    }
    */

    // empty - fn calibrate_ocode() {}
    //calibrate_ocode();

    // empty - fn set_rtc_dig_dbias() {}
    //set_rtc_dig_dbias();

    // we don't have external or spi memory
    //clock_control_init();

    // SYSTEM_CPU_WAIT_MODE_FORCE_ON == 0
    power_control_init();

    unsafe {
        // disable rtc interrupts
        rtc_cntl.int_ena().write(|w| w.bits(0));
        // clears rtc interrupts
        rtc_cntl.int_clr().write(|w| w.bits(u32::MAX));
    }
    // esp32c3 has no ULP (Ultra Low Power) coprocessor
    //regi2c::I2C_ULP_IR_FORCE_XPD_CK.write_field(0);
}

/// Perform power control related initialization
// 111
fn power_control_init() {
    let system = SYSTEM::regs();

    // ...

    // If SYSTEM_CPU_WAIT_MODE_FORCE_ON == 0,
    // the CPU clock will be closed when CPU enter WAITI mode.
    system
        .cpu_per_conf()
        .modify(|_, w| w.cpu_wait_mode_force_on().clear_bit());

    // ...
}

// 213
/// SOC Reset Reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum SocResetReason {
    /// Power on reset
    ///
    /// In ESP-IDF this value (0x01) can *also* be `ChipBrownOut` or
    /// `ChipSuperWdt`, however that is not really compatible with Rust-style
    /// enums.
    ChipPowerOn = 0x01,
    /// Software resets the digital core by RTC_CNTL_SW_SYS_RST
    CoreSw = 0x03,
    /// Deep sleep reset the digital core
    CoreDeepSleep = 0x05,
    /// Main watch dog 0 resets digital core
    CoreMwdt0 = 0x07,
    /// Main watch dog 1 resets digital core
    CoreMwdt1 = 0x08,
    /// RTC watch dog resets digital core
    CoreRtcWdt = 0x09,
    /// Main watch dog 0 resets CPU 0
    Cpu0Mwdt0 = 0x0B,
    /// Software resets CPU 0 by RTC_CNTL_SW_PROCPU_RST
    Cpu0Sw = 0x0C,
    /// RTC watch dog resets CPU 0
    Cpu0RtcWdt = 0x0D,
    /// VDD voltage is not stable and resets the digital core
    SysBrownOut = 0x0F,
    /// RTC watch dog resets digital core and rtc module
    SysRtcWdt = 0x10,
    /// Main watch dog 1 resets CPU 0
    Cpu0Mwdt1 = 0x11,
    /// Super watch dog resets the digital core and rtc module
    SysSuperWdt = 0x12,
    /// Glitch on clock resets the digital core and rtc module
    SysClkGlitch = 0x13,
    /// eFuse CRC error resets the digital core
    CoreEfuseCrc = 0x14,
    /// USB UART resets the digital core
    CoreUsbUart = 0x15,
    /// USB JTAG resets the digital core
    CoreUsbJtag = 0x16,
    /// Glitch on power resets the digital core
    CorePwrGlitch = 0x17,
}
