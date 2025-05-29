//! # Interrupt support
//!
//! ## Overview
//! This module routes one or more peripheral interrupt sources to any one
//! of the CPU’s peripheral interrupts.
//!
//! ## Configuration
//! Usually peripheral drivers offer a mechanism to register your interrupt
//! handler. e.g. the systimer offers `set_interrupt_handler` to register a
//! handler for a specific alarm. Other drivers might take an interrupt handler
//! as an optional parameter to their constructor.
//!
//! This is the preferred way to register handlers.
//!
//! There are additional ways to register interrupt handlers which are generally
//! only meant to be used in very special situations (mostly internal to the HAL
//! or the supporting libraries). Those are outside the scope of this
//! documentation.
//!
//! It is even possible, but not recommended, to bind an interrupt directly to a
//! CPU interrupt. This can offer lower latency, at the cost of more complexity
//! in the interrupt handler.
//!
//! We reserve a number of CPU interrupts, which cannot be used; see
//! [`RESERVED_INTERRUPTS`].

#[cfg(riscv)]
// 77
pub use self::riscv::*;

#[cfg(riscv)]
// 82
mod riscv;

#[no_mangle]
// 89
extern "C" fn EspDefaultHandler(_interrupt: crate::peripherals::Interrupt) {
    panic!("Unhandled interrupt: {:?}", _interrupt);
}

// 127
pub struct InterruptHandler {
    f: extern "C" fn(),
    prio: Priority,
}

// 132
impl InterruptHandler {
    /// Creates a new [InterruptHandler] which will call the given function at
    /// the given priority.
    pub const fn new(f: extern "C" fn(), prio: Priority) -> Self {
        Self { f, prio }
    }

    /// The function to be called
    #[inline]
    pub fn handler(&self) -> extern "C" fn() {
        self.f
    }

    /// Priority to be used when registering the interrupt
    #[inline]
    pub fn priority(&self) -> Priority {
        self.prio
    }
}

//#[cfg(not(any(large_intr_status, very_large_intr_status)))]
// 158
const STATUS_WORDS: usize = 2;

/// Representation of peripheral-interrupt status bits.
//#[derive(Clone, Copy, Default, Debug)]
#[derive(Clone, Copy, Default)]
// 163
pub struct InterruptStatus {
    status: [u32; STATUS_WORDS],
}

// 167
impl InterruptStatus {
    // 168
    const fn empty() -> Self {
        InterruptStatus {
            status: [0u32; STATUS_WORDS],
        }
    }

    //#[cfg(not(any(large_intr_status, very_large_intr_status)))]
    // 189
    const fn from(w0: u32, w1: u32) -> Self {
        Self { status: [w0, w1] }
    }

    /// Set the given interrupt status bit
    // 199
    pub fn set(&mut self, interrupt: u8) {
        self.status[interrupt as usize / 32] |= 1 << (interrupt % 32);
    }

    /// Return an iterator over the set interrupt status bits
    // 204
    pub fn iterator(&self) -> InterruptStatusIterator {
        InterruptStatusIterator {
            status: *self,
            idx: 0,
        }
    }
}

// 246
/// Iterator over set interrupt status bits
pub struct InterruptStatusIterator {
    status: InterruptStatus,
    idx: usize,
}

// 251
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
