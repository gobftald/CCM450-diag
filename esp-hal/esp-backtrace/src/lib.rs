#![no_std]

use esp_println::println;

const MAX_BACKTRACE_ADDRESSES: usize = 10;

#[cfg_attr(target_arch = "riscv32", path = "riscv.rs")]
pub mod arch;

#[cfg(feature = "panic-handler")]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("");
    println!("====================== PANIC ======================");

    #[cfg(not(feature = "defmt"))]
    println!("{}", info);

    #[cfg(feature = "defmt")]
    println!("{}", defmt::Display2Format(info));

    println!("");
    println!("Backtrace:");
    println!("");

    let backtrace = crate::arch::backtrace();
    #[cfg(target_arch = "riscv32")]
    if backtrace.iter().filter(|e| e.is_some()).count() == 0 {
        println!("No backtrace available - make sure to force frame-pointers. (see https://crates.io/crates/esp-backtrace)");
    }
    for addr in backtrace.into_iter().flatten() {
        println!("0x{:x}", addr - crate::arch::RA_OFFSET);
    }

    halt();
}

#[cfg(all(feature = "exception-handler", target_arch = "riscv32"))]
#[export_name = "ExceptionHandler"]
fn exception_handler(context: &arch::TrapFrame) -> ! {
    let mepc = context.pc;
    let code = context.mcause & 0xff;
    let mtval = context.mtval;

    if code == 14 {
        println!("");
        println!(
            "Stack overflow detected at 0x{:x} called by 0x{:x}",
            mepc, context.ra
        );
        println!("");
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

        println!(
            "Exception '{}' mepc=0x{:08x}, mtval=0x{:08x}",
            code, mepc, mtval
        );
        #[cfg(not(feature = "defmt"))]
        println!("{:x?}", context);

        #[cfg(feature = "defmt")]
        println!("{:?}", context);

        let backtrace = crate::arch::backtrace_internal(context.s0 as u32, 0);
        if backtrace.iter().filter(|e| e.is_some()).count() == 0 {
            println!("No backtrace available - make sure to force frame-pointers. (see https://crates.io/crates/esp-backtrace)");
        }
        for addr in backtrace.into_iter().flatten() {
            println!("0x{:x}", addr - crate::arch::RA_OFFSET);
        }
    }

    halt();
}

// Ensure that the address is in DRAM and that it is 16-byte aligned.
//
// Based loosely on the `esp_stack_ptr_in_dram` function from
// `components/esp_hw_support/include/esp_memory_utils.h` in ESP-IDF.
//
// Address ranges can be found in `components/soc/$CHIP/include/soc/soc.h` as
// `SOC_DRAM_LOW` and `SOC_DRAM_HIGH`.
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

fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}
