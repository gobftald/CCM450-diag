//! mcause register

/// mcause register
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 7
pub struct Mcause {
    bits: usize,
}

// 18
impl Mcause {
    /// Is trap cause an interrupt.
    // 48
    #[inline]
    pub fn is_interrupt(&self) -> bool {
        self.bits & (1 << (usize::BITS as usize - 1)) != 0
    }

    /// Is trap cause an exception.
    #[inline]
    // 54
    pub fn is_exception(&self) -> bool {
        !self.is_interrupt()
    }
}

// 59
read_csr_as!(Mcause, 0x342);
