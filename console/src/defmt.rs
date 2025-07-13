//! defmt global logger implementation.
// Implementation taken from esp-println (which taken from defmt-rtt,
// with a custom framing prefix)
// we also skip unnecessary critical-section logic (LockToken and write_bytes_in_cs)

#[cfg(feature = "critical-section")]
use critical_section::RestoreState;

use crate::usb_uart::*;

/// Global logger lock.
#[cfg(feature = "critical-section")]
static mut TAKEN: bool = false;

#[cfg(feature = "critical-section")]
static mut CS_RESTORE: RestoreState = RestoreState::invalid();

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

#[defmt::global_logger]
pub struct Logger;

#[allow(static_mut_refs)]
unsafe impl defmt::Logger for Logger {
    fn acquire() {
        unsafe {
            // safety: Must be paired with corresponding call to release(), see below
            let restore = critical_section::acquire();

            // safety: accessing the `static mut` is OK because we have acquired a critical
            // section.
            if TAKEN {
                crate::panic!("defmt logger taken reentrantly")
            }

            // safety: accessing the `static mut` is OK because we have acquired a critical
            // section.
            TAKEN = true;

            // safety: accessing the `static mut` is OK because we have acquired a critical
            // section.
            CS_RESTORE = restore;
        }

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
        unsafe {
            ENCODER.end_frame(print_wo_flush);

            Self::flush();

            #[cfg(feature = "critical-section")]
            {
                // safety: accessing the `static mut` is OK because we have acquired a critical
                // section.
                TAKEN = false;

                // safety: accessing the `static mut` is OK because we have acquired a critical
                // section.
                let restore = CS_RESTORE;

                // safety: Must be paired with corresponding call to acquire(), see above
                critical_section::release(restore);
            }
        }
    }

    unsafe fn flush() {
        unsafe { usb_uart_tx_flush() }
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because we have acquired a critical
        // section.
        unsafe { ENCODER.write(bytes, print_wo_flush) }
    }
}

// defmt::timestamp! defined in esp-hal/src/lib.rs
