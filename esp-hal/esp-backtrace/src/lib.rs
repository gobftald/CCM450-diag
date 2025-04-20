#![no_std]

#[allow(unused_imports)]
#[macro_use(println, core_println)]
extern crate console;

#[cfg(not(feature = "defmt"))]
use core::fmt::Write;

macro_rules! backtrace_println {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        println!($($arg)*);
        #[cfg(not(feature="defmt"))]
        core_println!($($arg)*);
    }
}

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

    #[inline]
    #[cfg(feature = "exception-handler")]
    // 24
    fn from_sp(sp: u32) -> Self {
        arch::backtrace_internal(sp, 0)
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
#[allow(unused_variables)]
#[panic_handler]
// 78
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    backtrace_println!("");
    backtrace_println!("====================== PANIC ======================");

    #[cfg(not(feature = "defmt"))]
    backtrace_println!("{}", info);
    #[cfg(not(feature = "defmt"))]
    backtrace_println!("");

    backtrace_println!("Backtrace:");

    let backtrace = Backtrace::capture();
    #[cfg(target_arch = "riscv32")]
    if backtrace.frames().is_empty() {
        backtrace_println!("No backtrace available - make sure to force frame-pointers. (see https://crates.io/crates/esp-backtrace)");
    }
    for frame in backtrace.frames() {
        backtrace_println!("0x{:x}", frame.program_counter());
    }

    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[cfg(all(feature = "exception-handler", target_arch = "riscv32"))]
#[export_name = "ExceptionHandler"]
// 121
fn exception_handler(context: &arch::TrapFrame) -> ! {
    let mepc = context.pc;
    let code = context.mcause & 0xff;
    let mtval = context.mtval;

    if code == 14 {
        backtrace_println!("");
        backtrace_println!(
            "Stack overflow detected at 0x{:x} called by 0x{:x}",
            mepc,
            context.ra
        );
        backtrace_println!("");
    } else {
        let code = match code {
            0 => "Instruction address misaligned",
            1 => "Instruction access fault",
            2 => "Illegal instruction",
            3 => "Breakpoint",
            4 => "Load address misaligned",
            5 => "Load access fault",
            6 => "Store/AMO address misaligned",
            7 => "Store/AMO access fault",
            8 => "Environment call from U-mode",
            9 => "Environment call from S-mode",
            10 => "Reserved",
            11 => "Environment call from M-mode",
            12 => "Instruction page fault",
            13 => "Load page fault",
            14 => "Reserved",
            15 => "Store/AMO page fault",
            _ => "UNKNOWN",
        };

        backtrace_println!("");
        backtrace_println!(
            "Exception '{}' mepc=0x{:08x}, mtval=0x{:08x}",
            code,
            mepc,
            mtval
        );

        backtrace_println!("{:?}", context);

        let backtrace = Backtrace::from_sp(context.s0 as u32);
        let frames = backtrace.frames();
        if frames.is_empty() {
            backtrace_println!("No backtrace available - make sure to force frame-pointers. (see https://crates.io/crates/esp-backtrace)");
        }
        for frame in backtrace.frames() {
            backtrace_println!("0x{:x}", frame.program_counter());
        }
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
