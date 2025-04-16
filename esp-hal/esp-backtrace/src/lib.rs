#![no_std]

const MAX_BACKTRACE_ADDRESSES: usize = 10;
pub struct Backtrace(pub(crate) heapless::Vec<BacktraceFrame, MAX_BACKTRACE_ADDRESSES>);

// 17
impl Backtrace {
    /// Captures a stack backtrace.
    #[inline]
    // 20
    pub fn capture() -> Self {
        arch::backtrace()
    }

    /// Returns the backtrace frames as a slice.
    #[inline]
    // 30
    pub fn frames(&self) -> &[BacktraceFrame] {
        &self.0
    }
}

// 37
pub struct BacktraceFrame {
    pub(crate) pc: usize,
}

// 41
impl BacktraceFrame {
    pub fn program_counter(&self) -> usize {
        self.pc - crate::arch::RA_OFFSET
    }
}

#[cfg_attr(target_arch = "riscv32", path = "riscv.rs")]
pub mod arch;

#[cfg(feature = "panic-handler")]
#[panic_handler]
// 78
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    use defmt::println;

    println!("");
    println!("====================== PANIC ======================");
    println!("Backtrace:");

    let backtrace = Backtrace::capture();
    #[cfg(target_arch = "riscv32")]
    if backtrace.frames().is_empty() {
        println!("No backtrace available - make sure to force frame-pointers. (see https://crates.io/crates/esp-backtrace)");
    }
    for frame in backtrace.frames() {
        println!("0x{:x}", frame.program_counter());
    }

    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

// Ensure that the address is in DRAM and that it is 16-byte aligned.
// 183
fn is_valid_ram_address(address: u32) -> bool {
    if (address & 0xF) != 0 {
        return false;
    }

    #[cfg(feature = "esp32c3")]
    if !(0x3FC8_0000..=0x3FCE_0000).contains(&address) {
        return false;
    }

    true
}
