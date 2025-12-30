//! Interrupt handling
//!
//! CPU interrupts 1 through 15 are reserved for each of the possible interrupt
//! priorities.
//!
//! On chips with a PLIC CPU interrupts 1,2,5,6,9 .. 19 are used.
//!
//! ```rust, ignore
//! interrupt1() => Priority::Priority1
//! interrupt2() => Priority::Priority2
//! ...
//! interrupt15() => Priority::Priority15
//! ```

// 15
#[cfg(feature = "rt")]
pub use esp_riscv_rt::TrapFrame;
use procmacros::ram;
use riscv::register::{mcause, mtvec};

// 20
#[cfg(not(plic))]
pub use self::classic::*;

// 24
pub use self::vectored::*;
use super::InterruptStatus;
use crate::{
    pac,
    peripherals::{INTERRUPT_CORE0, Interrupt},
    system::Cpu,
};

/// Interrupt Error
// 33
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The priority is not valid
    InvalidInterruptPriority,
    /// The CPU interrupt is a reserved interrupt
    CpuInterruptReserved,
}

/// Interrupt kind
// 43
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InterruptKind {
    /// Level interrupt
    Level,
    /// Edge interrupt
    Edge,
}

/// Enumeration of available CPU interrupts.
/// It is possible to create a handler for each of the interrupts. (e.g.
/// `interrupt3`)
// 54
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
// 123
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
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

// 161
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

// 173
impl TryFrom<u32> for Priority {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::None),
            1 => Ok(Priority::Priority1),
            2 => Ok(Priority::Priority2),
            3 => Ok(Priority::Priority3),
            4 => Ok(Priority::Priority4),
            5 => Ok(Priority::Priority5),
            6 => Ok(Priority::Priority6),
            7 => Ok(Priority::Priority7),
            8 => Ok(Priority::Priority8),
            9 => Ok(Priority::Priority9),
            10 => Ok(Priority::Priority10),
            11 => Ok(Priority::Priority11),
            12 => Ok(Priority::Priority12),
            13 => Ok(Priority::Priority13),
            14 => Ok(Priority::Priority14),
            15 => Ok(Priority::Priority15),
            _ => Err(Error::InvalidInterruptPriority),
        }
    }
}

// 199
impl TryFrom<u8> for Priority {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Priority::try_from(value as u32)
    }
}

/// The interrupts reserved by the HAL
// 208
#[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
pub static RESERVED_INTERRUPTS: &[u32] = PRIORITY_TO_INTERRUPT;

/// Enable an interrupt by directly binding it to a available CPU interrupt
///
/// ⚠️ This installs a *raw trap handler*, the `handler` user provides is written directly into the
/// CPU interrupt vector table. That means:
///
/// - Provided handler will be used as an actual trap-handler
/// - It is user's responsibility to:
///   - Save and restore all registers they use.
///   - Clear the interrupt source if necessary.
///   - Return using the `mret` instruction.
/// - The handler should be declared as naked function. The compiler will not insert a function
///   prologue/epilogue for the user, normal Rust `fn` will result in an error.
///
/// Unless you are sure that you need such low-level control to achieve the lowest possible latency,
/// you most likely want to use [`enable`] instead.
///
/// Trying using a reserved interrupt from [`RESERVED_INTERRUPTS`] will return
/// an error.
// 229
pub fn enable_direct(
    interrupt: Interrupt,
    level: Priority,
    cpu_interrupt: CpuInterrupt,
    handler: unsafe extern "C" fn(),
) -> Result<(), Error> {
    if RESERVED_INTERRUPTS.contains(&(cpu_interrupt as _)) {
        return Err(Error::CpuInterruptReserved);
    }
    if matches!(level, Priority::None) {
        return Err(Error::InvalidInterruptPriority);
    }
    unsafe {
        map(Cpu::current(), interrupt, cpu_interrupt);
        set_priority(Cpu::current(), cpu_interrupt, level);

        let mt = mtvec::read();

        assert_eq!(
            mt.trap_mode().into_usize(),
            mtvec::TrapMode::Vectored.into_usize()
        );

        let base_addr = mt.address() as usize;

        let int_slot = base_addr.wrapping_add((cpu_interrupt as usize) * 4);

        let instr = encode_jal_x0(handler as usize, int_slot)?;

        //if crate::debugger::debugger_connected() {
        core::ptr::write_volatile(int_slot as *mut u32, instr);
        /*
        } else {
            crate::debugger::DEBUGGER_LOCK.lock(|| {
                let wp = crate::debugger::clear_watchpoint(1);
                core::ptr::write_volatile(int_slot as *mut u32, instr);
                crate::debugger::restore_watchpoint(1, wp);
            });
        }
        */

        core::arch::asm!("fence.i");

        enable_cpu_interrupt(cpu_interrupt);
    }

    Ok(())
}

// helper: returns correctly encoded RISC-V `jal` instruction
// 276
fn encode_jal_x0(target: usize, pc: usize) -> Result<u32, Error> {
    let offset = (target as isize) - (pc as isize);

    const MIN: isize = -(1isize << 20);
    const MAX: isize = (1isize << 20) - 1;

    assert!(offset % 2 == 0 && (MIN..=MAX).contains(&offset));

    let imm = offset as u32;
    let imm20 = (imm >> 20) & 0x1;
    let imm10_1 = (imm >> 1) & 0x3ff;
    let imm11 = (imm >> 11) & 0x1;
    let imm19_12 = (imm >> 12) & 0xff;

    let instr = (imm20 << 31)
        | (imm19_12 << 12)
        | (imm11 << 20)
        | (imm10_1 << 21)
        // https://lhtin.github.io/01world/app/riscv-isa/?xlen=32&insn_name=jal
        | 0b1101111u32;

    Ok(instr)
}

/// Disable the given peripheral interrupt.
// 301
pub fn disable(core: Cpu, interrupt: Interrupt) {
    map_raw(core, interrupt, DISABLED_CPU_INTERRUPT)
}

/// Get status of peripheral interrupts
// 306
#[inline]
pub fn status(_core: Cpu) -> InterruptStatus {
    cfg_if::cfg_if! {
        if #[cfg(interrupts_status_registers = "3")] {
            InterruptStatus::from(
                INTERRUPT_CORE0::regs().core_0_intr_status(0).read().bits(),
                INTERRUPT_CORE0::regs().core_0_intr_status(1).read().bits(),
                INTERRUPT_CORE0::regs().core_0_intr_status(2).read().bits(),
            )
        } else if #[cfg(interrupts_status_registers = "4")] {
            InterruptStatus::from(
                INTERRUPT_CORE0::regs().core_0_intr_status(0).read().bits(),
                INTERRUPT_CORE0::regs().core_0_intr_status(1).read().bits(),
                INTERRUPT_CORE0::regs().core_0_intr_status(2).read().bits(),
                INTERRUPT_CORE0::regs().core_0_intr_status(3).read().bits(),
            )
        } else {
            InterruptStatus::from(
                INTERRUPT_CORE0::regs().core_0_intr_status(0).read().bits(),
                INTERRUPT_CORE0::regs().core_0_intr_status(1).read().bits(),
            )
        }
    }
}

/// Assign a peripheral interrupt to an CPU interrupt.
///
/// # Safety
///
/// Do not use CPU interrupts in the [`RESERVED_INTERRUPTS`].
// 336
pub unsafe fn map(core: Cpu, interrupt: Interrupt, which: CpuInterrupt) {
    map_raw(core, interrupt, which as u32)
}

// 340
pub fn map_raw(core: Cpu, interrupt: Interrupt, cpu_interrupt_number: u32) {
    let interrupt_number = interrupt as usize;

    match core {
        Cpu::ProCpu => {
            INTERRUPT_CORE0::regs()
                .core_0_intr_map(interrupt_number)
                .write(|w| unsafe { w.bits(cpu_interrupt_number) });
        }
        #[cfg(multi_core)]
        Cpu::AppCpu => {
            INTERRUPT_CORE1::regs()
                .core_1_intr_map(interrupt_number)
                .write(|w| unsafe { w.bits(cpu_interrupt_number) });
        }
    }
}

/// Get cpu interrupt assigned to peripheral interrupt
// 360
#[inline]
unsafe fn assigned_cpu_interrupt(interrupt: Interrupt) -> Option<CpuInterrupt> {
    let cpu_intr = INTERRUPT_CORE0::regs()
        .core_0_intr_map(interrupt as usize)
        .read()
        .bits();

    if cpu_intr > 0 && cpu_intr != DISABLED_CPU_INTERRUPT {
        Some(unsafe { core::mem::transmute::<u32, CpuInterrupt>(cpu_intr) })
    } else {
        None
    }
}

// 377
mod vectored {
    use super::*;
    use crate::interrupt::IsrCallback;

    // Setup interrupts ready for vectoring
    // 382
    #[doc(hidden)]
    pub(crate) unsafe fn init_vectoring() {
        for (num, prio) in PRIORITY_TO_INTERRUPT.iter().copied().zip(1..) {
            let which = unsafe { core::mem::transmute::<u32, CpuInterrupt>(num) };
            set_kind(Cpu::current(), which, InterruptKind::Level);
            unsafe {
                set_priority(
                    Cpu::current(),
                    which,
                    core::mem::transmute::<u8, Priority>(prio),
                );
                enable_cpu_interrupt(which);
            }
        }
    }

    /// Get the interrupts configured for the core at the given priority
    /// matching the given status
    // 400
    #[inline]
    pub(crate) fn configured_interrupts(
        core: Cpu,
        status: InterruptStatus,
        priority: Priority,
    ) -> InterruptStatus {
        unsafe {
            let mut res = InterruptStatus::empty();

            // interating when bit is 1 in the 64 bit interrupt status register
            for interrupt_nr in status.iterator() {
                // safety: cast is safe because of repr(u16)
                if let Some(cpu_interrupt) =
                    assigned_cpu_interrupt(core::mem::transmute::<u16, Interrupt>(
                        interrupt_nr as u16,
                    ))
                    && priority_by_core(core, cpu_interrupt) == priority
                {
                    res.set(interrupt_nr);
                }
            }
            res
        }
    }

    /// Enables a interrupt at a given priority
    ///
    /// Note that interrupts still need to be enabled globally for interrupts
    /// to be serviced.
    // 428
    pub fn enable(interrupt: Interrupt, level: Priority) -> Result<(), Error> {
        enable_on_cpu(Cpu::current(), interrupt, level)
    }

    // 432
    pub(crate) fn enable_on_cpu(
        cpu: Cpu,
        interrupt: Interrupt,
        level: Priority,
    ) -> Result<(), Error> {
        if matches!(level, Priority::None) {
            return Err(Error::InvalidInterruptPriority);
        }
        unsafe {
            let cpu_interrupt = core::mem::transmute::<u32, CpuInterrupt>(
                PRIORITY_TO_INTERRUPT[(level as usize) - 1],
            );
            map(cpu, interrupt, cpu_interrupt);
            enable_cpu_interrupt(cpu_interrupt);
        }
        Ok(())
    }

    /// Binds the given interrupt to the given handler.
    ///
    /// # Safety
    ///
    /// This will replace any previously bound interrupt handler
    // 455
    pub unsafe fn bind_interrupt(interrupt: Interrupt, handler: IsrCallback) {
        unsafe {
            let ptr =
                &pac::__EXTERNAL_INTERRUPTS[interrupt as usize]._handler as *const _ as *mut usize;

            //if crate::debugger::debugger_connected() {
            ptr.write_volatile(handler.raw_value());
            /*
            } else {
                crate::debugger::DEBUGGER_LOCK.lock(|| {
                    let wp = crate::debugger::clear_watchpoint(1);
                    ptr.write_volatile(handler.raw_value());
                    crate::debugger::restore_watchpoint(1, wp);
                });
            }
            */
        }
    }

    /// Returns the currently bound interrupt handler.
    // 473
    pub fn bound_handler(interrupt: Interrupt) -> Option<IsrCallback> {
        unsafe {
            let addr = pac::__EXTERNAL_INTERRUPTS[interrupt as usize]._handler as usize;
            if addr == 0 {
                return None;
            }

            Some(IsrCallback::from_raw(addr))
        }
    }
}

// 486
#[cfg(not(plic))]
mod classic {
    use super::{CpuInterrupt, InterruptKind, Priority};
    use crate::{peripherals::INTERRUPT_CORE0, system::Cpu};

    // 490
    #[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    pub(super) static DISABLED_CPU_INTERRUPT: u32 = 0;

    // 493
    #[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    pub(super) static PRIORITY_TO_INTERRUPT: &[u32] =
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    // First element is not used, just there to avoid a -1 in the interrupt handler.
    // 498
    #[cfg_attr(place_switch_tables_in_ram, unsafe(link_section = ".rwtext"))]
    pub(super) static INTERRUPT_TO_PRIORITY: [Priority; 16] = [
        Priority::None,
        Priority::Priority1,
        Priority::Priority2,
        Priority::Priority3,
        Priority::Priority4,
        Priority::Priority5,
        Priority::Priority6,
        Priority::Priority7,
        Priority::Priority8,
        Priority::Priority9,
        Priority::Priority10,
        Priority::Priority11,
        Priority::Priority12,
        Priority::Priority13,
        Priority::Priority14,
        Priority::Priority15,
    ];

    /// Enable a CPU interrupt
    ///
    /// # Safety
    ///
    /// Make sure there is an interrupt handler registered.
    // 523
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
    // 534
    pub fn set_kind(_core: Cpu, which: CpuInterrupt, kind: InterruptKind) {
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
    // 558
    pub unsafe fn set_priority(_core: Cpu, which: CpuInterrupt, priority: Priority) {
        let intr = INTERRUPT_CORE0::regs();
        intr.cpu_int_pri(which as usize)
            .write(|w| unsafe { w.map().bits(priority as u8) });
    }

    /// Clear a CPU interrupt
    #[inline]
    // 566
    pub fn clear(_core: Cpu, which: CpuInterrupt) {
        unsafe {
            let cpu_interrupt_number = which as usize;
            let intr = INTERRUPT_CORE0::regs();
            intr.cpu_int_clear()
                .write(|w| w.bits(1 << cpu_interrupt_number));
        }
    }

    /// Get interrupt priority
    // 576
    #[inline]
    pub(super) fn priority_by_core(_core: Cpu, cpu_interrupt: CpuInterrupt) -> Priority {
        priority(cpu_interrupt)
    }

    /// Get interrupt priority - can be called by assembly code as well
    #[inline]
    // 583
    pub(super) fn priority(cpu_interrupt: CpuInterrupt) -> Priority {
        let intr = INTERRUPT_CORE0::regs();
        unsafe {
            core::mem::transmute::<u8, Priority>(
                intr.cpu_int_pri(cpu_interrupt as usize).read().map().bits(),
            )
        }
    }

    /// Get the current run level (the level below which interrupts are masked).
    // 593
    pub fn current_runlevel() -> Priority {
        let intr = INTERRUPT_CORE0::regs();
        let prev_interrupt_priority = intr.cpu_int_thresh().read().bits().saturating_sub(1) as u8;

        unwrap!(Priority::try_from(prev_interrupt_priority))
    }

    /// Changes the current run level (the level below which interrupts are
    /// masked), and returns the previous run level.
    ///
    /// # Safety
    ///
    /// This function must only be used to raise the runlevel and to restore it
    /// to a previous value. It must not be used to arbitrarily lower the
    /// runlevel.
    // 608
    pub(crate) unsafe fn change_current_runlevel(level: Priority) -> Priority {
        let prev_interrupt_priority = current_runlevel();

        // The CPU responds to interrupts `>= level`, but we want to also disable
        // interrupts at `level` so we set the threshold to `level + 1`.
        INTERRUPT_CORE0::regs()
            .cpu_int_thresh()
            .write(|w| unsafe { w.bits(level as u32 + 1) });

        prev_interrupt_priority
    }
}

#[cfg(feature = "rt")]
mod rt {
    use esp_riscv_rt::TrapFrame;

    use super::*;

    /// # Safety
    ///
    /// This function is called from an assembly trap handler.
    // 772
    #[unsafe(link_section = ".trap.rust")]
    #[unsafe(export_name = "_start_trap_rust_hal")]
    unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
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

        unsafe extern "C" {
            fn ExceptionHandler(tf: *mut TrapFrame);
        }

        unsafe {
            ExceptionHandler(trap_frame);
        }
    }

    // 789
    #[unsafe(no_mangle)]
    // 230
    pub fn _setup_interrupts() {
        unsafe extern "C" {
            static _vector_table: u32;
        }

        // disable all known interrupts
        // at least after the 2nd stage bootloader there are some interrupts enabled
        // (e.g. UART)
        for peripheral_interrupt in 0..255 {
            crate::peripherals::Interrupt::try_from(peripheral_interrupt)
                .map(|intr| {
                    #[cfg(multi_core)]
                    disable(Cpu::AppCpu, intr);
                    disable(Cpu::ProCpu, intr);
                })
                .ok();
        }

        unsafe {
            let vec_table = (&_vector_table as *const u32).addr();
            mtvec::write({
                let mut mtvec = mtvec::Mtvec::from_bits(0);
                mtvec.set_trap_mode(mtvec::TrapMode::Vectored);
                mtvec.set_address(vec_table);
                mtvec
            });

            crate::interrupt::init_vectoring();
        }
    }

    // 826
    #[unsafe(no_mangle)]
    #[ram]
    unsafe fn handle_interrupts(cpu_intr: CpuInterrupt) {
        let core = Cpu::current();
        let status = status(core);

        // this has no effect on level interrupts, but the interrupt may be an edge one
        // so we clear it anyway
        clear(core, cpu_intr);

        // it should be >= then the current runlevel
        let prio = INTERRUPT_TO_PRIORITY[cpu_intr as usize];
        // get all pending peripheral irq at this priority [where priority(1-15) = irq(1-15)]
        let configured_interrupts = vectored::configured_interrupts(core, status, prio);

        // Change the current runlevel so that interrupt handlers can access the correct runlevel.
        // old runlevel saved into level
        let level = unsafe { change_current_runlevel(prio) };

        // When nesting is possible, we run the nestable interrupts first. This ensures that we
        // don't violate assumptions made by non-nestable handlers.

        if prio != Priority::max() {
            for interrupt_nr in configured_interrupts.iterator() {
                let handler =
                    unsafe { pac::__EXTERNAL_INTERRUPTS[interrupt_nr as usize]._handler } as usize;
                let nested = (handler & 1) == 0;
                if nested {
                    let handler: fn() =
                        unsafe { core::mem::transmute::<usize, fn()>(handler & !1) };

                    unsafe { riscv::interrupt::nested(handler) };
                }
            }
        }

        for interrupt_nr in configured_interrupts.iterator() {
            let handler =
                unsafe { pac::__EXTERNAL_INTERRUPTS[interrupt_nr as usize]._handler } as usize;
            let not_nested = (handler & 1) == 1;
            if not_nested || prio == Priority::max() {
                let handler: fn() = unsafe { core::mem::transmute::<usize, fn()>(handler & !1) };

                handler();
            }
        }

        unsafe { change_current_runlevel(level) };
    }
}
