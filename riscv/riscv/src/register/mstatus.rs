//! mstatus register

use crate::bits::{bf_extract, bf_insert};

/// mstatus register
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mstatus {
    bits: usize,
}

// 84
impl Mstatus {
    /// Machine Interrupt Enable
    #[inline]
    // 109
    pub fn mie(&self) -> bool {
        bf_extract(self.bits, 3, 1) != 0
    }

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

// 518
set!(0x300);
clear!(0x300);

// 527
set_clear_csr!(
    /// Machine Interrupt Enable
    , set_mie, clear_mie, 1 << 3);
