#[cfg(not(any(esp32c6, esp32h2)))]
// 4
use peripherals::SYSTEM as SystemPeripheral;

// 6
use crate::{
    hal::{
        interrupt::{self, TrapFrame},
        peripherals::{self, Interrupt},
        riscv,
    },
    preempt_builtin::{task_switch, timer::setup_timebase},
    TimeBase,
};

// 16
pub(crate) fn setup_timer(timer: TimeBase) {
    // make sure the scheduling won't start before everything is setup
    riscv::interrupt::disable();

    setup_timebase(timer);
}

// 23
pub(crate) fn setup_multitasking() {
    unwrap!(interrupt::enable(
        Interrupt::FROM_CPU_INTR2,
        interrupt::Priority::Priority1,
    ));

    unsafe {
        riscv::interrupt::enable();
    }
}

#[unsafe(no_mangle)]
// 39
extern "C" fn FROM_CPU_INTR2(trap_frame: &mut TrapFrame) {
    // clear FROM_CPU_INTR3
    SystemPeripheral::regs()
        //.cpu_intr_from_cpu_2()
        .cpu_intr_from_cpu(2)
        //.modify(|_, w| w.cpu_intr_from_cpu_2().clear_bit());
        .modify(|_, w| w.cpu_intr().clear_bit());

    task_switch(trap_frame);
}

// 48
pub(crate) fn yield_task() {
    SystemPeripheral::regs()
        //.cpu_intr_from_cpu_2()
        .cpu_intr_from_cpu(2)
        //.modify(|_, w| w.cpu_intr_from_cpu_2().set_bit());
        .modify(|_, w| w.cpu_intr().set_bit());
}
