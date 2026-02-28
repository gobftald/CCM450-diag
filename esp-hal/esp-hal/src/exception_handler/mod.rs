use crate::trapframe::TrapFrame;

#[cfg(riscv)]
#[unsafe(no_mangle)]
unsafe extern "C" fn ExceptionHandler(context: &TrapFrame) -> ! {
    let mepc = riscv::register::mepc::read();
    let code = riscv::register::mcause::read().code();
    let mtval = riscv::register::mtval::read();

    unsafe extern "C" {
        static mut __stack_chk_guard: u32;
    }

    if code == 14 {
        panic!(
            "Stack overflow detected at 0x{:x}, possibly called by 0x{:x}",
            mepc, context.ra
        );
    }

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

    panic!(
        "Exception '{}' mepc=0x{:08x}, mtval=0x{:08x}\n{:?}",
        code, mepc, mtval, context
    );
}