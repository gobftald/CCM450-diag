//! Interrupt handling
//!
//! CPU interrupts 1 through 15 are reserved for each of the possible interrupt
//! priorities.

// 16
pub use esp_riscv_rt::TrapFrame;
use riscv::register::{mcause, mtvec};

//#[cfg(not(plic))]
//19
pub use self::classic::*;

// 22
pub use self::vectored::*;
use super::InterruptStatus;

// 24
use crate::{
    pac,
    peripherals::{Interrupt, INTERRUPT_CORE0},
};

/// Interrupt kind
//#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 42
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
// 55
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
// 124
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

// 159
impl Priority {
    /// Maximum interrupt priority
    pub const fn max() -> Priority {
        Priority::Priority15
    }

    /// Minimum interrupt priority
    pub const fn min() -> Priority {
        Priority::Priority1
    }
}

/// # Safety
///
/// This function is called from an assembly trap handler.
#[link_section = ".trap.rust"]
#[export_name = "_start_trap_rust_hal"]
// 216
pub unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
    // if assert failed and if build-std-features = ["panic_immediate_abort"]
    // this is a forever loop
    // assert makes panic, but panic at "panic_immediate_abort" concludes unimp
    // exception, which comes back here again
    //
    // in defmt case this makes the error message repeated forever
    // in no defmt case the forever loop is running, but no error messages
    //
    // fortunately this is only a rare  (theoretical) error case
    assert!(
        mcause::read().is_exception(),
        "Arrived into _start_trap_rust_hal but mcause is not an exception!"
    );

    extern "C" {
        fn ExceptionHandler(tf: *mut TrapFrame);
    }
    ExceptionHandler(trap_frame);
}

#[unsafe(no_mangle)]
// 230
pub fn _setup_interrupts() {
    unsafe extern "C" {
        unsafe static _vector_table: *const u32;
    }

    unsafe {
        // disable all known interrupts
        // at least after the 2nd stage bootloader there are some interrupts enabled
        // (e.g. UART)
        for peripheral_interrupt in 0..core::mem::variant_count::<Interrupt>() as u8 {
            // Don't use `Interrupt::try_from`. It takes 0xf8 space in .rodata
            /*
            Interrupt::try_from(peripheral_interrupt)
                .map(|intr| {
                    disable(intr);
                })
                .ok();
            */
            disable(peripheral_interrupt);
        }

        let vec_table = &_vector_table as *const _ as usize;
        mtvec::write(vec_table, mtvec::TrapMode::Vectored);

        crate::interrupt::init_vectoring();
    }
}

/// Disable the given peripheral interrupt.
// 287
//pub fn disable(interrupt: Interrupt) {
pub fn disable(interrupt: u8) {
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

/// Get status of peripheral interrupts
#[inline]
// 302
pub fn status() -> InterruptStatus {
    InterruptStatus::from(
        INTERRUPT_CORE0::regs().intr_status_reg_0().read().bits(),
        INTERRUPT_CORE0::regs().intr_status_reg_1().read().bits(),
    )
}

/// Get cpu interrupt assigned to peripheral interrupt
#[inline]
// 349
unsafe fn assigned_cpu_interrupt(interrupt: Interrupt) -> Option<CpuInterrupt> {
    let interrupt_number = interrupt as isize;
    let intr_map_base = crate::soc::registers::INTERRUPT_MAP_BASE as *mut u32;

    let cpu_intr = unsafe { intr_map_base.offset(interrupt_number).read_volatile() };
    if cpu_intr > 0 && cpu_intr != DISABLED_CPU_INTERRUPT {
        Some(unsafe {
            core::mem::transmute::<u32, CpuInterrupt>(cpu_intr - EXTERNAL_INTERRUPT_OFFSET)
        })
    } else {
        None
    }
}

// 369
mod vectored {
    use procmacros::ram;

    use super::*;

    // Setup interrupts ready for vectoring
    // 376
    pub(crate) unsafe fn init_vectoring() {
        for (prio, num) in PRIORITY_TO_INTERRUPT.iter().enumerate() {
            unsafe {
                set_kind(
                    //Cpu::current(),
                    core::mem::transmute::<u32, CpuInterrupt>(*num as u32),
                    InterruptKind::Level,
                );
                set_priority(
                    //Cpu::current(),
                    core::mem::transmute::<u32, CpuInterrupt>(*num as u32),
                    core::mem::transmute::<u8, Priority>((prio as u8) + 1),
                );
                enable_cpu_interrupt(core::mem::transmute::<u32, CpuInterrupt>(*num as u32));
            }
        }
    }

    /// Get the interrupts configured for the core at the given priority
    /// matching the given status
    #[inline]
    // 397
    fn configured_interrupts(status: InterruptStatus, priority: Priority) -> InterruptStatus {
        unsafe {
            let mut res = InterruptStatus::empty();

            // interating when bit is 1 in the 64 bit interrupt status register
            for interrupt_nr in status.iterator() {
                // safety: cast is safe because of repr(u16)
                if let Some(cpu_interrupt) =
                    assigned_cpu_interrupt(core::mem::transmute::<u16, Interrupt>(
                        interrupt_nr as u16,
                    ))
                {
                    if priority_by_core(cpu_interrupt) == priority {
                        res.set(interrupt_nr);
                    }
                }
            }
            res
        }
    }

    /// Binds the given interrupt to the given handler.
    ///
    /// # Safety
    ///
    /// This will replace any previously bound interrupt handler
    // 452
    pub unsafe fn bind_interrupt(interrupt: Interrupt, handler: unsafe extern "C" fn()) {
        unsafe {
            let ptr = &pac::__EXTERNAL_INTERRUPTS[interrupt as usize]._handler as *const _
                as *mut unsafe extern "C" fn();
            ptr.write_volatile(handler);
        }
    }

    #[unsafe(no_mangle)]
    #[ram]
    // 474
    unsafe fn handle_interrupts(cpu_intr: CpuInterrupt, context: &mut TrapFrame) {
        // this has no effect on level interrupts, but the interrupt may be an edge one
        // so we clear it anyway
        clear(cpu_intr);

        // it is 1:15 - 1:15
        let priority = INTERRUPT_TO_PRIORITY[cpu_intr as usize];
        let prio: Priority = unsafe { core::mem::transmute(priority) };

        // get all pending peripheral irq at this priority [where priority(1-15) = irq(1-15)]
        let configured_interrupts = configured_interrupts(status(), prio);

        for interrupt_nr in configured_interrupts.iterator() {
            // Don't use `Interrupt::try_from`. It's slower and placed in flash
            let interrupt: Interrupt = unsafe { core::mem::transmute(interrupt_nr as u16) };
            unsafe {
                handle_interrupt(interrupt, context);
            }
        }
    }

    // 496
    #[inline(always)]
    unsafe fn handle_interrupt(interrupt: Interrupt, save_frame: &mut TrapFrame) {
        unsafe extern "C" {
            // defined in each hal
            fn EspDefaultHandler(interrupt: Interrupt);
        }

        let handler = unsafe { pac::__EXTERNAL_INTERRUPTS[interrupt as usize]._handler };

        if core::ptr::eq(
            handler as *const _,
            EspDefaultHandler as *const unsafe extern "C" fn(),
        ) {
            unsafe { EspDefaultHandler(interrupt) };
        } else {
            let handler: fn(&mut TrapFrame) = unsafe {
                core::mem::transmute::<unsafe extern "C" fn(), fn(&mut TrapFrame)>(handler)
            };
            handler(save_frame);
        }
    }

    // The compiler generates quite unfortunate code for
    // ```rust,ignore
    // #[no_mangle]
    // #[ram]
    // unsafe fn interrupt1(context: &mut TrapFrame) {
    //    handle_interrupts(CpuInterrupt::Interrupt1, context)
    // }
    // ```
    //
    // Resulting in
    // ```asm,ignore
    // interrupt1:
    // add	sp,sp,-16
    // sw	ra,12(sp)
    // sw	s0,8(sp)
    // add	s0,sp,16
    // mv	a1,a0
    // li	a0,1
    // lw	ra,12(sp)
    // lw	s0,8(sp)
    // add	sp,sp,16
    // auipc	t1,0x0
    // jr	handle_interrupts
    // ```
    //
    // We can do better manually - use Rust again once/if that changes
    // 543
    macro_rules! interrupt_handler {
        ($num:literal) => {
            core::arch::global_asm! {
                concat!(
                r#"
                    .section .rwtext, "ax"
                    .global interrupt"#,$num,r#"

                interrupt"#,$num,r#":
                    mv a1, a0
                    li a0,"#,$num,r#"
                    j handle_interrupts
                "#
            )
            }
        };
    }

    // 561
    interrupt_handler!(1);
    interrupt_handler!(2);
    interrupt_handler!(3);
    interrupt_handler!(4);
    interrupt_handler!(5);
    interrupt_handler!(6);
    interrupt_handler!(7);
    interrupt_handler!(8);
    interrupt_handler!(9);
    interrupt_handler!(10);
    interrupt_handler!(11);
    interrupt_handler!(12);
    interrupt_handler!(13);
    interrupt_handler!(14);
    interrupt_handler!(15);
}

//#[cfg(not(plic))]
// 588
mod classic {
    use super::{CpuInterrupt, InterruptKind, Priority};
    use crate::peripherals::INTERRUPT_CORE0;

    //#[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    #[unsafe(link_section = ".rwtext")]
    // 593
    pub(super) static DISABLED_CPU_INTERRUPT: u32 = 0;

    //#[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    #[unsafe(link_section = ".rwtext")]
    // 596
    pub(super) static EXTERNAL_INTERRUPT_OFFSET: u32 = 0;

    //#[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    #[unsafe(link_section = ".rwtext")]
    // 599
    pub(super) static PRIORITY_TO_INTERRUPT: &[usize] =
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    // 604
    // First element is not used, just there to avoid a -1 in the interrupt handler.
    //#[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    #[unsafe(link_section = ".rwtext")]
    pub(super) static INTERRUPT_TO_PRIORITY: [u8; 16] =
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    /// Enable a CPU interrupt
    ///
    /// # Safety
    ///
    /// Make sure there is an interrupt handler registered.
    pub unsafe fn enable_cpu_interrupt(which: CpuInterrupt) {
        let cpu_interrupt_number = which as isize;
        let intr = INTERRUPT_CORE0::regs();
        intr.cpu_int_enable()
            .modify(|r, w| unsafe { w.bits((1 << cpu_interrupt_number) | r.bits()) });
    }

    /// Set the interrupt kind (i.e. level or edge) of an CPU interrupt
    ///
    /// The vectored interrupt handler will take care of clearing edge interrupt
    /// bits.
    // 623
    pub fn set_kind(which: CpuInterrupt, kind: InterruptKind) {
        unsafe {
            let intr = INTERRUPT_CORE0::regs();
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
    // 647
    pub unsafe fn set_priority(which: CpuInterrupt, priority: Priority) {
        let intr = INTERRUPT_CORE0::regs();
        intr.cpu_int_pri(which as usize)
            .write(|w| unsafe { w.map().bits(priority as u8) });
    }

    /// Clear a CPU interrupt
    #[inline]
    // 655
    pub fn clear(which: CpuInterrupt) {
        unsafe {
            let cpu_interrupt_number = which as usize;
            let intr = INTERRUPT_CORE0::regs();
            intr.cpu_int_clear()
                .write(|w| w.bits(1 << cpu_interrupt_number));
        }
    }

    /// Get interrupt priority
    #[inline]
    // 666
    pub(super) fn priority_by_core(cpu_interrupt: CpuInterrupt) -> Priority {
        unsafe { priority(cpu_interrupt) }
    }

    /// Get interrupt priority - can be called by assembly code as well
    #[inline]
    // 672
    pub(super) unsafe extern "C" fn priority(cpu_interrupt: CpuInterrupt) -> Priority {
        let intr = INTERRUPT_CORE0::regs();
        unsafe {
            core::mem::transmute::<u8, Priority>(
                intr.cpu_int_pri(cpu_interrupt as usize).read().map().bits(),
            )
        }
    }
}
