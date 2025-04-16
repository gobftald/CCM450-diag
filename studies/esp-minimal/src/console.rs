#![allow(dead_code)]

#[cfg(feature = "defmt")]
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

#[cfg(feature = "defmt")]
#[defmt::global_logger]
pub struct Logger;

// using cooperative scheduler (like embassy) and not or carefully
// implementing log/print in ISRs can eliminate critical sections
#[cfg(feature = "defmt")]
#[allow(static_mut_refs)]
unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // Write a non-UTF8 sequence to indicate the start of a defmt frame.
        // We need this to distinguish defmt frames from other data that
        // might be written to the printer.
        print_wo_flush(&[0xFF, 0x00]);

        // safety: accessing the `static mut` is OK because
        // we ensure not using nested defmt calls
        unsafe { ENCODER.start_frame(print_wo_flush) }
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because
        // we ensure not using nested defmt calls
        ENCODER.end_frame(print_wo_flush);

        Self::flush();
    }

    unsafe fn flush() {
        usb_uart_tx_flush();
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because
        // we ensure not using nested defmt calls
        ENCODER.write(bytes, print_wo_flush);
    }
}

extern "C" {
    fn usb_uart_tx_one_char(char: u8);
    fn usb_uart_tx_flush();
}

#[cfg(feature = "console")]
pub fn print_wo_flush(bytes: &[u8]) {
    unsafe {
        for byte in bytes {
            usb_uart_tx_one_char(*byte);
        }
    }
}

#[cfg(feature = "console")]
pub fn print(bytes: &[u8]) {
    print_wo_flush(bytes);
    unsafe { usb_uart_tx_flush() }
}
