#[cfg(riscv)]
pub use self::riscv::*;

#[cfg(riscv)]
mod riscv;

#[cfg(not(any(large_intr_status, very_large_intr_status)))]
// 122
const STATUS_WORDS: usize = 2;

/// Representation of peripheral-interrupt status bits.
#[derive(Clone, Copy, Default, Debug)]
// 126
pub struct InterruptStatus {
    status: [u32; STATUS_WORDS],
}

// 130
impl InterruptStatus {
    const fn empty() -> Self {
        InterruptStatus {
            status: [0u32; STATUS_WORDS],
        }
    }

    #[cfg(not(any(large_intr_status, very_large_intr_status)))]
    // 152
    const fn from(w0: u32, w1: u32) -> Self {
        Self { status: [w0, w1] }
    }

    /// Set the given interrupt status bit
    // 162
    pub fn set(&mut self, interrupt: u16) {
        self.status[interrupt as usize / 32] |= 1 << (interrupt as u32 % 32);
    }

    /// Return an iterator over the set interrupt status bits
    // 167
    pub fn iterator(&self) -> InterruptStatusIterator {
        InterruptStatusIterator {
            status: *self,
            idx: 0,
        }
    }
}

/// Iterator over set interrupt status bits
// 209
pub struct InterruptStatusIterator {
    status: InterruptStatus,
    idx: usize,
}

// 214x
impl Iterator for InterruptStatusIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.idx..STATUS_WORDS {
            if self.status.status[i] != 0 {
                let bit = self.status.status[i].trailing_zeros();
                self.idx = i;
                self.status.status[i] &= !1 << bit;
                return Some((bit + 32 * i as u32) as u8);
            }
        }
        self.idx = usize::MAX;
        None
    }
}
