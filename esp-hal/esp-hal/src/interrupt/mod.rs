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
//! in the interrupt handler. See the `direct_vectoring.rs` example
//!
//! We reserve a number of CPU interrupts, which cannot be used; see
//! [`RESERVED_INTERRUPTS`].
//!
//! ## Examples
//!
//! ### Using the peripheral driver to register an interrupt handler
//!
//! ```rust, no_run
//! # {before_snippet}
//! let mut sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
//! critical_section::with(|cs| {
//!     sw_int
//!         .software_interrupt0
//!         .set_interrupt_handler(swint0_handler);
//!     SWINT0
//!         .borrow_ref_mut(cs)
//!         .replace(sw_int.software_interrupt0);
//! });
//!
//! critical_section::with(|cs| {
//!     if let Some(swint) = SWINT0.borrow_ref(cs).as_ref() {
//!         swint.raise();
//!     }
//! });
//! #
//! # loop {}
//! # }
//!
//! # use core::cell::RefCell;
//! #
//! # use critical_section::Mutex;
//! # use esp_hal::interrupt::software::{SoftwareInterrupt, SoftwareInterruptControl};
//! # use esp_hal::interrupt::Priority;
//! # use esp_hal::interrupt::InterruptHandler;
//! #
//! static SWINT0: Mutex<RefCell<Option<SoftwareInterrupt<0>>>> = Mutex::new(RefCell::new(None));
//!
//! #[handler(priority = Priority::Priority1)]
//! fn swint0_handler() {
//!     println!("SW interrupt0");
//!     critical_section::with(|cs| {
//!         if let Some(swint) = SWINT0.borrow_ref(cs).as_ref() {
//!             swint.reset();
//!         }
//!     });
//! }
//! ```

// 73
use core::{num::NonZeroUsize, ops::BitAnd};

// 75
#[cfg(riscv)]
pub use self::riscv::*;
#[cfg(xtensa)]
pub use self::xtensa::*;

// 80
#[cfg(riscv)]
mod riscv;
#[cfg(xtensa)]
mod xtensa;

// 85
pub mod software;

// 87
#[cfg(feature = "rt")]
#[unsafe(no_mangle)]
extern "C" fn EspDefaultHandler() {
    panic!("Unhandled interrupt on {:?}", crate::system::Cpu::current());
}

/// Default (unhandled) interrupt handler
// 94
pub const DEFAULT_INTERRUPT_HANDLER: InterruptHandler = InterruptHandler::new(
    {
        unsafe extern "C" {
            fn EspDefaultHandler();
        }

        unsafe {
            core::mem::transmute::<unsafe extern "C" fn(), extern "C" fn()>(EspDefaultHandler)
        }
    },
    Priority::min(),
);

/// Trait implemented by drivers which allow the user to set an
/// [InterruptHandler]
// 109
pub trait InterruptConfigurable: crate::private::Sealed {
    #[cfg_attr(
        not(multi_core),
        doc = "Registers an interrupt handler for the peripheral."
    )]
    #[cfg_attr(
        multi_core,
        doc = "Registers an interrupt handler for the peripheral on the current core."
    )]
    #[doc = ""]
    /// Note that this will replace any previously registered interrupt
    /// handlers. Some peripherals offer a shared interrupt handler for
    /// multiple purposes. It's the users duty to honor this.
    ///
    /// You can restore the default/unhandled interrupt handler by using
    /// [DEFAULT_INTERRUPT_HANDLER]
    fn set_interrupt_handler(&mut self, handler: InterruptHandler);
}

/// Represents an ISR callback function
// 129
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IsrCallback {
    f: NonZeroUsize,
}

// 135
impl IsrCallback {
    /// Construct a new callback from the callback function.
    pub fn new(f: extern "C" fn()) -> Self {
        // a valid fn pointer is non zero
        Self {
            f: unwrap!(NonZeroUsize::new(f as usize)),
        }
    }

    /// Construct a new callback from the callback function and the nested flag.
    // 145
    pub(crate) fn new_with_nested(f: extern "C" fn(), nested: bool) -> Self {
        // a valid fn pointer is non zero
        let f = unwrap!(NonZeroUsize::new(f as usize | !nested as usize));
        Self { f }
    }

    /// Construct a new callback from a raw value.
    ///
    /// # Panics
    ///
    /// Passing zero is invalid and results in a panic.
    // 156
    pub fn from_raw(f: usize) -> Self {
        Self {
            f: unwrap!(NonZeroUsize::new(f)),
        }
    }

    /// Returns the raw value of the callback.
    ///
    /// Don't just cast this to function and call it - it might be misaligned.
    // 165
    pub fn raw_value(self) -> usize {
        self.f.into()
    }

    /// The callback function.
    ///
    /// This is aligned and can be called.
    // 172
    pub fn aligned_ptr(self) -> extern "C" fn() {
        unsafe { core::mem::transmute::<usize, extern "C" fn()>(Into::<usize>::into(self.f) & !1) }
    }
}

// 177
impl PartialEq for IsrCallback {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::fn_addr_eq(self.aligned_ptr(), other.aligned_ptr())
    }
}

/// An interrupt handler
#[cfg_attr(
    multi_core,
    doc = "**Note**: Interrupts are handled on the core they were setup on, if a driver is initialized on core 0, and moved to core 1, core 0 will still handle the interrupt."
)]
// 188
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterruptHandler {
    f: extern "C" fn(),
    prio: Priority,
    #[cfg(riscv)]
    nested: bool,
}

// 197
impl InterruptHandler {
    /// Creates a new [InterruptHandler] which will call the given function at
    /// the given priority.
    pub const fn new(f: extern "C" fn(), prio: Priority) -> Self {
        Self {
            f,
            prio,
            #[cfg(riscv)]
            nested: true,
        }
    }

    /// Creates a new [InterruptHandler] which will call the given function at
    /// the given priority with disabled interrupt nesting.
    ///
    /// Usually higher priority interrupts get served while handling an interrupt.
    /// Using this the interrupt handler won't get preempted by higher priority interrupts.
    // 214
    #[cfg(riscv)]
    pub fn new_not_nested(f: extern "C" fn(), prio: Priority) -> Self {
        Self {
            f,
            prio,
            nested: false,
        }
    }

    /// The Isr callback.
    // 224
    #[inline]
    pub fn handler(&self) -> IsrCallback {
        cfg_if::cfg_if! {
            if #[cfg(riscv)] {
                let nested = self.nested;
            } else {
                let nested = true;
            }
        }

        IsrCallback::new_with_nested(self.f, nested)
    }

    /// Priority to be used when registering the interrupt
    // 238
    #[inline]
    pub fn priority(&self) -> Priority {
        self.prio
    }
}

// 244
const STATUS_WORDS: usize = property!("interrupts.status_registers");

/// Representation of peripheral-interrupt status bits.
// 247
#[derive(Clone, Copy, Default, Debug)]
pub struct InterruptStatus {
    status: [u32; STATUS_WORDS],
}

// 252
impl InterruptStatus {
    const fn empty() -> Self {
        InterruptStatus {
            status: [0u32; STATUS_WORDS],
        }
    }

    // 259
    #[cfg(interrupts_status_registers = "3")]
    const fn from(w0: u32, w1: u32, w2: u32) -> Self {
        Self {
            status: [w0, w1, w2],
        }
    }

    // 266
    #[cfg(interrupts_status_registers = "4")]
    const fn from(w0: u32, w1: u32, w2: u32, w3: u32) -> Self {
        Self {
            status: [w0, w1, w2, w3],
        }
    }

    // 273
    #[cfg(interrupts_status_registers = "2")]
    const fn from(w0: u32, w1: u32) -> Self {
        Self { status: [w0, w1] }
    }

    /// Is the given interrupt bit set
    // 278
    pub fn is_set(&self, interrupt: u8) -> bool {
        (self.status[interrupt as usize / 32] & (1 << (interrupt % 32))) != 0
    }

    /// Set the given interrupt status bit
    // 284
    pub fn set(&mut self, interrupt: u8) {
        self.status[interrupt as usize / 32] |= 1 << (interrupt % 32);
    }

    /// Return an iterator over the set interrupt status bits
    // 289
    pub fn iterator(&self) -> InterruptStatusIterator {
        InterruptStatusIterator {
            status: *self,
            idx: 0,
        }
    }
}

// 297
impl BitAnd for InterruptStatus {
    type Output = InterruptStatus;

    fn bitand(self, rhs: Self) -> Self::Output {
        #[cfg(interrupts_status_registers = "3")]
        return Self::Output {
            status: [
                self.status[0] & rhs.status[0],
                self.status[1] & rhs.status[1],
                self.status[2] & rhs.status[2],
            ],
        };

        #[cfg(interrupts_status_registers = "4")]
        return Self::Output {
            status: [
                self.status[0] & rhs.status[0],
                self.status[1] & rhs.status[1],
                self.status[2] & rhs.status[2],
                self.status[3] & rhs.status[3],
            ],
        };

        #[cfg(interrupts_status_registers = "2")]
        return Self::Output {
            status: [
                self.status[0] & rhs.status[0],
                self.status[1] & rhs.status[1],
            ],
        };
    }
}

/// Iterator over set interrupt status bits
// 330
pub struct InterruptStatusIterator {
    status: InterruptStatus,
    idx: usize,
}

// 336
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
