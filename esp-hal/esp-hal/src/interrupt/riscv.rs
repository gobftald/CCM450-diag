//! Interrupt handling
//!
//! CPU interrupts 1 through 15 are reserved for each of the possible interrupt
//! priorities.
//!
//! ```rust, ignore
//! interrupt1() => Priority::Priority1
//! interrupt2() => Priority::Priority2
//! ...
//! interrupt15() => Priority::Priority15
//! ```

pub use esp_riscv_rt::TrapFrame;
use riscv::register::{mcause, mtvec};

#[cfg(not(plic))]
pub use self::classic::*;
pub(crate) use self::vectored::*;
use crate::{peripherals::Interrupt, Cpu};

/// Interrupt kind
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 41
pub enum InterruptKind {
    /// Level interrupt
    Level,
    /// Edge interrupt
    Edge,
}

/// Enumeration of available CPU interrupts.
/// It is possible to create a handler for each of the interrupts. (e.g.
/// `interrupt3`)
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 54
pub enum CpuInterrupt {
    /// Interrupt number 1.
    Interrupt1 = 1,
    /// Interrupt number 2.
    Interrupt2,
    /// Interrupt number 3.
    Interrupt3,
    /// Interrupt number 4.
    Interrupt4,
    /// Interrupt number 5.
    Interrupt5,
    /// Interrupt number 6.
    Interrupt6,
    /// Interrupt number 7.
    Interrupt7,
    /// Interrupt number 8.
    Interrupt8,
    /// Interrupt number 9.
    Interrupt9,
    /// Interrupt number 10.
    Interrupt10,
    /// Interrupt number 11.
    Interrupt11,
    /// Interrupt number 12.
    Interrupt12,
    /// Interrupt number 13.
    Interrupt13,
    /// Interrupt number 14.
    Interrupt14,
    /// Interrupt number 15.
    Interrupt15,
    /// Interrupt number 16.
    Interrupt16,
    /// Interrupt number 17.
    Interrupt17,
    /// Interrupt number 18.
    Interrupt18,
    /// Interrupt number 19.
    Interrupt19,
    /// Interrupt number 20.
    Interrupt20,
    /// Interrupt number 21.
    Interrupt21,
    /// Interrupt number 22.
    Interrupt22,
    /// Interrupt number 23.
    Interrupt23,
    /// Interrupt number 24.
    Interrupt24,
    /// Interrupt number 25.
    Interrupt25,
    /// Interrupt number 26.
    Interrupt26,
    /// Interrupt number 27.
    Interrupt27,
    /// Interrupt number 28.
    Interrupt28,
    /// Interrupt number 29.
    Interrupt29,
    /// Interrupt number 30.
    Interrupt30,
    /// Interrupt number 31.
    Interrupt31,
}

/// Interrupt priority levels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
// 123
pub enum Priority {
    /// No priority.
    None = 0,
    /// Priority level 1.
    Priority1,
    /// Priority level 2.
    Priority2,
    /// Priority level 3.
    Priority3,
    /// Priority level 4.
    Priority4,
    /// Priority level 5.
    Priority5,
    /// Priority level 6.
    Priority6,
    /// Priority level 7.
    Priority7,
    /// Priority level 8.
    Priority8,
    /// Priority level 9.
    Priority9,
    /// Priority level 10.
    Priority10,
    /// Priority level 11.
    Priority11,
    /// Priority level 12.
    Priority12,
    /// Priority level 13.
    Priority13,
    /// Priority level 14.
    Priority14,
    /// Priority level 15.
    Priority15,
}

/// # Safety
///
/// This function is called from an assembly trap handler.
#[doc(hidden)]
#[link_section = ".trap.rust"]
#[export_name = "_start_trap_rust_hal"]
// 179
pub unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
    assert!(
        mcause::read().is_exception(),
        "Arrived into _start_trap_rust_hal but mcause is not an exception!"
    );
    extern "C" {
        fn ExceptionHandler(tf: *mut TrapFrame);
    }
    // either the DefaultExceptionHandler from esp-risv-rt or
    // the external ExceptionHandler function e.g. from esp-backtrace
    ExceptionHandler(trap_frame);
}

#[doc(hidden)]
#[no_mangle]
// 192
pub fn _setup_interrupts() {
    extern "C" {
        static _vector_table: *const u32;
    }

    unsafe {
        // disable all known interrupts
        // at least after the 2nd stage bootloader there are some interrupts enabled
        // (e.g. UART)
        for peripheral_interrupt in 0..255 {
            // esp32c3 has only 0..62
            crate::soc::peripherals::Interrupt::try_from(peripheral_interrupt)
                .map(|intr| {
                    #[cfg(multi_core)]
                    disable(Cpu::AppCpu, intr);
                    disable(Cpu::ProCpu, intr);
                })
                .ok();
        }

        let vec_table = &_vector_table as *const _ as usize;
        mtvec::write(vec_table, mtvec::TrapMode::Vectored);

        crate::interrupt::init_vectoring();
    }
}

/// Disable the given peripheral interrupt.
// 249
pub fn disable(_core: Cpu, interrupt: Interrupt) {
    unsafe {
        let interrupt_number = interrupt as isize;
        let intr_map_base = crate::soc::registers::INTERRUPT_MAP_BASE as *mut u32;

        // set to 0 to disable the peripheral interrupt on chips with an interrupt
        // controller other than PLIC use the disabled interrupt 31 otherwise
        intr_map_base
            .offset(interrupt_number)
            .write_volatile(DISABLED_CPU_INTERRUPT);
    }
}

// 357
mod vectored {
    use super::*;

    // Setup interrupts ready for vectoring
    // 364
    pub(crate) unsafe fn init_vectoring() {
        for (prio, num) in PRIORITY_TO_INTERRUPT.iter().enumerate() {
            set_kind(
                crate::get_core(),
                core::mem::transmute::<u32, CpuInterrupt>(*num as u32),
                InterruptKind::Level,
            );
            set_priority(
                crate::get_core(),
                core::mem::transmute::<u32, CpuInterrupt>(*num as u32),
                core::mem::transmute::<u8, Priority>((prio as u8) + 1),
            );
            enable_cpu_interrupt(core::mem::transmute::<u32, CpuInterrupt>(*num as u32));
        }
    }
}

#[cfg(not(plic))]
// 548
mod classic {
    use super::{CpuInterrupt, InterruptKind, Priority};
    use crate::Cpu;

    // 552
    pub(super) const DISABLED_CPU_INTERRUPT: u32 = 0;

    // 516
    pub(super) const PRIORITY_TO_INTERRUPT: &[usize] =
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    /// Enable a CPU interrupt
    ///
    /// # Safety
    ///
    /// Make sure there is an interrupt handler registered.
    // 567
    pub unsafe fn enable_cpu_interrupt(which: CpuInterrupt) {
        let cpu_interrupt_number = which as isize;
        let intr = &*crate::peripherals::INTERRUPT_CORE0::PTR;
        intr.cpu_int_enable()
            .modify(|r, w| w.bits((1 << cpu_interrupt_number) | r.bits()));
    }

    /// Set the interrupt kind (i.e. level or edge) of an CPU interrupt
    ///
    /// The vectored interrupt handler will take care of clearing edge interrupt
    /// bits.
    // 578
    pub fn set_kind(_core: Cpu, which: CpuInterrupt, kind: InterruptKind) {
        unsafe {
            let intr = &*crate::peripherals::INTERRUPT_CORE0::PTR;
            let cpu_interrupt_number = which as isize;

            let interrupt_type = match kind {
                InterruptKind::Level => 0,
                InterruptKind::Edge => 1,
            };
            intr.cpu_int_type().modify(|r, w| {
                w.bits(
                    r.bits() & !(1 << cpu_interrupt_number)
                        | (interrupt_type << cpu_interrupt_number),
                )
            });
        }
    }

    /// Set the priority level of an CPU interrupt
    ///
    /// # Safety
    ///
    /// Great care must be taken when using this function; avoid changing the
    /// priority of interrupts 1 - 15.
    // 596
    pub unsafe fn set_priority(_core: Cpu, which: CpuInterrupt, priority: Priority) {
        let intr = &*crate::peripherals::INTERRUPT_CORE0::PTR;
        intr.cpu_int_pri(which as usize)
            .write(|w| w.map().bits(priority as u8));
    }

    #[no_mangle]
    #[link_section = ".trap"]
    // 635
    pub(super) unsafe extern "C" fn _handle_priority() -> u32 {
        use super::mcause;
        let interrupt_id: usize = mcause::read().code(); // MSB is whether its exception or interrupt.
        let intr = &*crate::peripherals::INTERRUPT_CORE0::PTR;
        let interrupt_priority = intr
            .cpu_int_pri(0)
            .as_ptr()
            .add(interrupt_id)
            .read_volatile();

        let prev_interrupt_priority = intr.cpu_int_thresh().read().bits();
        if interrupt_priority < 15 {
            // leave interrupts disabled if interrupt is of max priority.
            intr.cpu_int_thresh()
                .write(|w| w.bits(interrupt_priority + 1)); // set the prio threshold to 1 more than current interrupt prio
            unsafe {
                riscv::interrupt::enable();
            }
        }
        prev_interrupt_priority
    }

    #[no_mangle]
    #[link_section = ".trap"]
    // 658
    pub(super) unsafe extern "C" fn _restore_priority(stored_prio: u32) {
        riscv::interrupt::disable();
        let intr = &*crate::peripherals::INTERRUPT_CORE0::PTR;
        intr.cpu_int_thresh().write(|w| w.bits(stored_prio));
    }
}
