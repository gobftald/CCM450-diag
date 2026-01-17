#![no_std]

pub mod result;

use result::Result;

/// Trait for enums of target-specific interrupt numbers.
///
/// This trait should be implemented by a peripheral access crate (PAC) on its enum of available
/// interrupts for a specific device. Alternatively, the `riscv` crate provides a default
/// implementation for the RISC-V ISA. Each variant must convert to a `usize` of its interrupt number.
///
/// # Safety
///
/// * This trait must only be implemented on the `riscv` crate or on a PAC of a RISC-V target.
/// * This trait must only be implemented on enums of interrupts.
/// * Each enum variant must represent a distinct value (no duplicates are permitted),
/// * Each enum variant must always return the same value (do not change at runtime).
/// * All the interrupt numbers must be less than or equal to `MAX_INTERRUPT_NUMBER`.
/// * `MAX_INTERRUPT_NUMBER` must coincide with the highest allowed interrupt number.
// 46
pub unsafe trait InterruptNumber: Copy {
    /// Highest number assigned to an interrupt source.
    const MAX_INTERRUPT_NUMBER: usize;

    /// Converts an interrupt source to its corresponding number.
    fn number(self) -> usize;

    /// Tries to convert a number to a valid interrupt.
    fn from_number(value: usize) -> Result<Self>;
}

/// Marker trait for enums of target-specific core interrupt numbers.
///
/// Core interrupts are interrupts are retrieved from the `mcause` CSR. Usually, vectored mode is
/// only available for core interrupts. The `riscv` crate provides a default implementation for
/// the RISC-V ISA. However, a PAC may override the default implementation if the target has a
/// different interrupt numbering scheme (e.g., ESP32C3).
///
/// # Safety
///
/// Each enum variant must represent a valid core interrupt number read from the `mcause` CSR.
// 67
pub unsafe trait CoreInterruptNumber: InterruptNumber {}
