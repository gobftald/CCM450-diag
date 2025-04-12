#![no_std]
#![no_main]

// since we config 'build-std-features = ["panic_immediate_abort"]'
// this handler will be never called, but the compiler insists on it
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[esp_hal::main]
fn main() -> ! {
    usb_print(b"01234567890123456789012345678901234567890123456789012345678\n");

    //loop {} // jmp (2 bytes)
    panic!(); // unimp (2 bytes)
}

fn usb_print(bytes: &[u8]) {
    extern "C" {
        fn usb_uart_tx_one_char(char: u8);
        fn usb_uart_tx_flush();
    }

    unsafe {
        for byte in bytes {
            usb_uart_tx_one_char(*byte);
        }

        usb_uart_tx_flush();
    }
}
