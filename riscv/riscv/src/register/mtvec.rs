//! mtvec register

/// mtvec register
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(dead_code)]
// 5
pub struct Mtvec {
    bits: usize,
}

/// Trap mode
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 11
pub enum TrapMode {
    Direct = 0,
    Vectored = 1,
}

// 43
write_csr!(0x305);

/// Writes the CSR
#[inline]
// 47
pub unsafe fn write(addr: usize, mode: TrapMode) {
    let bits = addr + mode as usize;
    _write(bits);
}
