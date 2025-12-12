use portable_atomic::{AtomicBool, Ordering};

const SERIAL_JTAG_FIFO_REG: usize = 0x6004_3000;
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

pub fn print_wo_flush(bytes: &[u8]) {
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

    for byte in bytes {
        if fifo_full() {
            fifo_flush();

            // Wait for the FIFO to clear, we have more data to shift out.
            if !wait_for_flush() {
                return;
            }
        }
        fifo_write(*byte);
    }
}

pub fn print(bytes: &[u8]) {
    print_wo_flush(bytes);
    fifo_flush()
}

#[allow(unused)]
pub fn flush() {
    fifo_flush()
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s.as_bytes());
        Ok(())
    }
}
