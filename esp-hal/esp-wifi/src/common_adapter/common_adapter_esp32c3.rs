// 3
use crate::hal::peripherals::{APB_CTRL, LPWR};

// 8
const SOC_PHY_DIG_REGS_MEM_SIZE: usize = 21 * 4;

// 10
static mut SOC_PHY_DIG_REGS_MEM: [u8; SOC_PHY_DIG_REGS_MEM_SIZE] = [0u8; SOC_PHY_DIG_REGS_MEM_SIZE];

// 12
static mut G_PHY_DIGITAL_REGS_MEM: *mut u32 = core::ptr::null_mut();

// 16
pub(crate) fn enable_wifi_power_domain() {
    const SYSTEM_WIFIBB_RST: u32 = 1 << 0;
    const SYSTEM_FE_RST: u32 = 1 << 1;
    const SYSTEM_WIFIMAC_RST: u32 = 1 << 2;
    const SYSTEM_BTBB_RST: u32 = 1 << 3; // Bluetooth Baseband
    const SYSTEM_BTMAC_RST: u32 = 1 << 4; // deprecated
    const SYSTEM_RW_BTMAC_RST: u32 = 1 << 9; // Bluetooth MAC
    const SYSTEM_RW_BTMAC_REG_RST: u32 = 1 << 11; // Bluetooth MAC Regsiters
    const SYSTEM_BTBB_REG_RST: u32 = 1 << 13; // Bluetooth Baseband Registers

    const MODEM_RESET_FIELD_WHEN_PU: u32 = SYSTEM_WIFIBB_RST
        | SYSTEM_FE_RST
        | SYSTEM_WIFIMAC_RST
        | SYSTEM_BTBB_RST
        | SYSTEM_BTMAC_RST
        | SYSTEM_RW_BTMAC_RST
        | SYSTEM_RW_BTMAC_REG_RST
        | SYSTEM_BTBB_REG_RST;

    LPWR::regs()
        .dig_pwc()
        .modify(|_, w| w.wifi_force_pd().clear_bit());

    APB_CTRL::regs()
        .wifi_rst_en()
        .modify(|r, w| unsafe { w.bits(r.bits() | MODEM_RESET_FIELD_WHEN_PU) });

    APB_CTRL::regs()
        .wifi_rst_en()
        .modify(|r, w| unsafe { w.bits(r.bits() & !MODEM_RESET_FIELD_WHEN_PU) });

    LPWR::regs()
        .dig_iso()
        .modify(|_, w| w.wifi_force_iso().clear_bit());
}

// 52
pub(crate) fn phy_mem_init() {
    unsafe {
        G_PHY_DIGITAL_REGS_MEM = core::ptr::addr_of_mut!(SOC_PHY_DIG_REGS_MEM).cast();
    }
}
