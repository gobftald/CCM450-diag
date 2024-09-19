#![no_std]

// MUST be the first module
pub mod fmt;

#[cfg(riscv)]
// 142
pub use esp_riscv_rt::entry;

#[cfg(any(dport, interrupt_core0, interrupt_core1))]
// 186
pub mod interrupt;

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
