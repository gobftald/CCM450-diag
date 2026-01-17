//! Interrupts

pub mod machine;

#[cfg(not(feature = "s-mode"))]
pub use machine::*;

/// Trap Cause.
///
/// This enum represents the cause of a trap. It can be either an interrupt or an exception.
/// The [`mcause`](crate::register::mcause::Mcause::cause) and
/// [`scause`](crate::register::scause::Scause::cause) registers return a value of this type.
/// However, the trap cause is represented as raw numbers. To get a target-specific trap cause,
/// use [`Trap::try_into`] with your target-specific M-Mode or S-Mode trap cause types.
///
/// # Example
///
/// In targets that comply with the RISC-V standard, you can use the standard
/// [`Interrupt`] and [`Exception`] enums to represent the trap cause:
///
/// ```no_run
/// use riscv::interrupt::{Trap, Interrupt, Exception};
/// use riscv::register::mcause;
///
/// let raw_trap: Trap<usize, usize> = mcause::read().cause();
/// let standard_trap: Trap<Interrupt, Exception> = raw_trap.try_into().unwrap();
/// ```
///
/// Targets that do not comply with the RISC-V standard usually have their own interrupt and exceptions.
/// You can find these types in the target-specific PAC. If it has been generated with `svd2rust`,
/// you can use the `pac::interrupt::CoreInterrupt` and `pac::interrupt::Exception` enums:
///
/// ```ignore,no_run
/// use riscv::interrupt::Trap;
/// use pac::interrupt::{CoreInterrupt, Exception}; // pac is the target-specific PAC
///
/// let standard_trap: Trap<CoreInterrupt, Exception> = pac::interrupt::cause();
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap<I, E> {
    Interrupt(I),
    Exception(E),
}
