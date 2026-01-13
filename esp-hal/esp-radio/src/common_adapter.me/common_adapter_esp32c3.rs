// 1
use portable_atomic::{AtomicU32, Ordering};

// 3
use crate::{
    binary::include::{phy_dig_reg_backup, phy_wakeup_init},
    hal::peripherals::{APB_CTRL, LPWR},
};

// 8
const SOC_PHY_DIG_REGS_MEM_SIZE: usize = 21 * 4;

// 10
static mut SOC_PHY_DIG_REGS_MEM: [u8; SOC_PHY_DIG_REGS_MEM_SIZE] = [0u8; SOC_PHY_DIG_REGS_MEM_SIZE];
static mut G_IS_PHY_CALIBRATED: bool = false;
static mut G_PHY_DIGITAL_REGS_MEM: *mut u32 = core::ptr::null_mut();
static mut S_IS_PHY_REG_STORED: bool = false;
static PHY_ACCESS_REF: AtomicU32 = AtomicU32::new(0);

// 52
pub(crate) fn phy_mem_init() {
    unsafe {
        G_PHY_DIGITAL_REGS_MEM = core::ptr::addr_of_mut!(SOC_PHY_DIG_REGS_MEM).cast();
    }
}

// 58
pub(crate) unsafe fn bbpll_en_usb() {
    #[cfg(phy_enable_usb)]
    {
        unsafe extern "C" {
            fn phy_bbpll_en_usb(param: bool);
        }

        unsafe {
            phy_bbpll_en_usb(true);
        }
    }
}

// 71
pub(crate) unsafe fn phy_enable() {
    let count = PHY_ACCESS_REF.fetch_add(1, Ordering::SeqCst);
    if count == 0 {
        critical_section::with(|_| {
            unsafe {
                super::phy_enable_clock();
            }

            if unsafe { !G_IS_PHY_CALIBRATED } {
                super::phy_calibrate();
                unsafe { G_IS_PHY_CALIBRATED = true };
            } else {
                unsafe {
                    phy_wakeup_init();
                }
                phy_digital_regs_load();
            }

            trace!("PHY ENABLE");
        });
    } else {
        info!("PHY ENABLE warning: it is called more than once");
    }
}

// 131
fn phy_digital_regs_load() {
    unsafe {
        if S_IS_PHY_REG_STORED && !G_PHY_DIGITAL_REGS_MEM.is_null() {
            phy_dig_reg_backup(false, G_PHY_DIGITAL_REGS_MEM);
        }
    }
}
