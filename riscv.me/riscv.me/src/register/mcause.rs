//! mcause register

pub use crate::interrupt::Trap;

read_only_csr! {
    /// `mcause` register
    Mcause: 0x342,
    mask: usize::MAX,
}

#[cfg(target_arch = "riscv32")]
read_only_csr_field! {
    Mcause,
    /// Returns the `code` field.
    code: [0:30],
}

#[cfg(target_arch = "riscv32")]
read_only_csr_field! {
    Mcause,
    /// Is the trap cause an interrupt.
    is_interrupt: 31,
}

impl Mcause {
    #[inline]
    pub fn cause(&self) -> Trap<usize, usize> {
        if self.is_interrupt() {
            Trap::Interrupt(self.code())
        } else {
            Trap::Exception(self.code())
        }
    }

    /// Is trap cause an exception.
    #[inline]
    pub fn is_exception(&self) -> bool {
        !self.is_interrupt()
    }
}
