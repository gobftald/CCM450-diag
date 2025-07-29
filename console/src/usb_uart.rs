unsafe extern "C" {
    fn usb_uart_tx_one_char(char: u8);
    pub fn usb_uart_tx_flush();
}

pub fn print_wo_flush(bytes: &[u8]) {
    unsafe {
        for byte in bytes {
            usb_uart_tx_one_char(*byte);
        }
    }
}

pub fn print(bytes: &[u8]) {
    print_wo_flush(bytes);
    unsafe { usb_uart_tx_flush() }
}

pub fn flush() {
    unsafe { usb_uart_tx_flush() }
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s.as_bytes());
        Ok(())
    }
}
