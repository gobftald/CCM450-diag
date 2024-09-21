//! mstatus register

use crate::bits::bf_insert;

/// mstatus register
#[derive(Clone, Copy, Debug)]
// 8
pub struct Mstatus {
    bits: usize,
}

// 84
impl Mstatus {
    /// Update Machine Interrupt Enable
    ///
    /// Note this updates a previously read [`Mstatus`] value, but does not
    /// affect the mstatus CSR itself. See [`set_mie`]/[`clear_mie`] to directly
    /// update the CSR.
    #[inline]
    // 119
    pub fn set_mie(&mut self, mie: bool) {
        self.bits = bf_insert(self.bits, 3, 1, mie as usize);
    }
}

set!(0x300);
clear!(0x300);

set_clear_csr!(
    /// Machine Interrupt Enable
    , set_mie, clear_mie, 1 << 3);
