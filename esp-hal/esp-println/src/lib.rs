#![no_std]

#[cfg(feature = "log")]
pub mod logger;

/// Prints to the selected output, with a newline.
#[cfg(not(feature = "no-op"))]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            writeln!($crate::Printer, $($arg)*).ok();
        }
    }};
}

/// Prints to the selected output.
#[cfg(not(feature = "no-op"))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        {
            use core::fmt::Write;
            write!($crate::Printer, $($arg)*).ok();
        }
    }};
}

/// Prints to the configured output, with a newline.
#[cfg(feature = "no-op")]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{}};
}

/// Prints to the configured output.
#[cfg(feature = "no-op")]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{}};
}

/// Prints and returns the value of a given expression for quick and dirty
/// debugging.
// implementation adapted from `std::dbg`
#[macro_export]
macro_rules! dbg {
        // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `println!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}]", ::core::file!(), ::core::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}",
                    ::core::file!(), ::core::line!(), ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

/// The printer that is used by the `print!` and `println!` macros.
pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Printer::write_bytes(s.as_bytes());
        Ok(())
    }
}

impl Printer {
    /// Writes a byte slice to the configured output.
    pub fn write_bytes(bytes: &[u8]) {
        with(|| {
            PrinterImpl::write_bytes_assume_cs(bytes);
            PrinterImpl::flush();
        })
    }
}

#[cfg(feature = "jtag-serial")]
type PrinterImpl = serial_jtag_printer::Printer;

#[cfg(all(
    feature = "jtag-serial",
    any(
        feature = "esp32c3",
        feature = "esp32c6",
        feature = "esp32h2",
        feature = "esp32p4",
        feature = "esp32s3"
    )
))]
mod serial_jtag_printer {
    use portable_atomic::{AtomicBool, Ordering};
    pub struct Printer;

    #[cfg(feature = "esp32c3")]
    const SERIAL_JTAG_FIFO_REG: usize = 0x6004_3000;
    #[cfg(feature = "esp32c3")]
    const SERIAL_JTAG_CONF_REG: usize = 0x6004_3004;

    /// A previous wait has timed out. We use this flag to avoid blocking
    /// forever if there is no host attached.
    static TIMED_OUT: AtomicBool = AtomicBool::new(false);

    fn fifo_flush() {
        let conf = SERIAL_JTAG_CONF_REG as *mut u32;
        unsafe { conf.write_volatile(0b001) };
    }

    fn fifo_full() -> bool {
        let conf = SERIAL_JTAG_CONF_REG as *mut u32;
        unsafe { conf.read_volatile() & 0b010 == 0b000 }
    }

    fn fifo_write(byte: u8) {
        let fifo = SERIAL_JTAG_FIFO_REG as *mut u32;
        unsafe { fifo.write_volatile(byte as u32) }
    }

    fn wait_for_flush() -> bool {
        const TIMEOUT_ITERATIONS: usize = 50_000;

        // Wait for some time for the FIFO to clear.
        let mut timeout = TIMEOUT_ITERATIONS;
        while fifo_full() {
            if timeout == 0 {
                TIMED_OUT.store(true, Ordering::Relaxed);
                return false;
            }
            timeout -= 1;
        }

        true
    }

    impl Printer {
        pub fn write_bytes_assume_cs(bytes: &[u8]) {
            if fifo_full() {
                // The FIFO is full. Let's see if we can progress.

                if TIMED_OUT.load(Ordering::Relaxed) {
                    // Still wasn't able to drain the FIFO. Let's assume we won't be able to, and
                    // don't queue up more data.
                    // This is important so we don't block forever if there is no host attached.
                    return;
                }

                // Give the fifo some time to drain.
                if !wait_for_flush() {
                    return;
                }
            } else {
                // Reset the flag - we managed to clear our FIFO.
                TIMED_OUT.store(false, Ordering::Relaxed);
            }

            for &b in bytes {
                if fifo_full() {
                    fifo_flush();

                    // Wait for the FIFO to clear, we have more data to shift out.
                    if !wait_for_flush() {
                        return;
                    }
                }
                fifo_write(b);
            }
        }

        pub fn flush() {
            fifo_flush();
        }
    }
}

#[inline]
fn with<R>(f: impl FnOnce() -> R) -> R {
    #[cfg(feature = "critical-section")]
    return critical_section::with(|_| f());

    #[cfg(not(feature = "critical-section"))]
    f()
}
