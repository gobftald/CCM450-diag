#![no_std]

// MUST be the first module
pub mod fmt;

#[cfg(riscv)]
// 142
pub use esp_riscv_rt::entry;

// 157
pub use self::soc::peripherals;

#[cfg(any(dport, interrupt_core0, interrupt_core1))]
// 186
pub mod interrupt;

// 201
pub mod peripheral;

// 248
// The `soc` module contains chip-specific implementation details and should not
// be directly exposed.
mod soc;

#[cfg(riscv)]
#[no_mangle]
// 258
extern "C" fn EspDefaultHandler(_interrupt: peripherals::Interrupt) {
    panic!("Unhandled interrupt: {:?}", _interrupt);
}

// 278
pub(crate) mod private {
    pub trait Sealed {}
}

/// Available CPU cores
///
/// The actual number of available cores depends on the target.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
// 346
pub enum Cpu {
    /// The first core
    ProCpu = 0,
    /// The second core
    #[cfg(multi_core)]
    AppCpu = 1,
}

/// Which core the application is currently executing on
#[inline(always)]
// 360
pub fn get_core() -> Cpu {
    // This works for both RISCV and Xtensa because both
    // get_raw_core functions return zero, _or_ something
    // greater than zero; 1 in the case of RISCV and 0x2000
    // in the case of Xtensa.
    match get_raw_core() {
        0 => Cpu::ProCpu,
        #[cfg(all(multi_core, riscv))]
        1 => Cpu::AppCpu,
        #[cfg(all(multi_core, xtensa))]
        0x2000 => Cpu::AppCpu,
        _ => unreachable!(),
    }
}

/// Returns the raw value of the mhartid register.
///
/// Safety: This method should never return UNUSED_THREAD_ID_VALUE
#[cfg(riscv)]
#[inline(always)]
fn get_raw_core() -> usize {
    #[cfg(multi_core)]
    {
        riscv::register::mhartid::read()
    }

    #[cfg(not(multi_core))]
    0
}

#[cfg(riscv)]
#[export_name = "hal_main"]
// 698
fn hal_main(a0: usize, a1: usize, a2: usize) -> ! {
    extern "Rust" {
        // This symbol will be provided by the user via `#[entry]`
        fn main(a0: usize, a1: usize, a2: usize) -> !;
    }

    unsafe { main(a0, a1, a2) }
}

pub fn rom_usb_print(bytes: &[u8]) {
    extern "C" {
        fn usb_uart_tx_flush();
        fn usb_uart_tx_one_char(u8: u8);
    }
    unsafe {
        for byte in bytes {
            usb_uart_tx_flush();
            usb_uart_tx_one_char(*byte);
        }
        usb_uart_tx_flush();
    }
}
