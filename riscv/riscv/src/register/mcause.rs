//! mcause register

/// mcause register
#[derive(Clone, Copy, Debug)]
// 5
pub struct Mcause {
    bits: usize,
}

// 119
impl Mcause {
    /// Is trap cause an interrupt.
    #[inline]
    // 144
    pub fn is_interrupt(&self) -> bool {
        self.bits & (1 << (usize::BITS as usize - 1)) != 0
    }

    /// Is trap cause an exception.
    #[inline]
    // 150
    pub fn is_exception(&self) -> bool {
        !self.is_interrupt()
    }
}

// 155
read_csr_as!(Mcause, 0x342);
