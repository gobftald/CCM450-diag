use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr::{self, NonNull};
use core::slice;

use super::{
    alloc::{Allocator, Global, Layout},
    boxed::Box,
};

//use super::alloc::handle_alloc_error;
use alloc_crate::alloc::handle_alloc_error;

/// The error type for `try_reserve` methods.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 19
pub struct TryReserveError {
    kind: TryReserveErrorKind,
}

/// Details of the allocation that caused a `TryReserveError`
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 32
#[allow(dead_code)] // until we use AllocError
pub enum TryReserveErrorKind {
    /// Error due to the computed capacity exceeding the collection's maximum
    /// (usually `isize::MAX` bytes).
    CapacityOverflow,

    /// The memory allocator returned an error
    AllocError {
        /// The layout of allocation request that failed
        layout: Layout,
        non_exhaustive: (),
    },
}

// 47
use crate::raw_vec::TryReserveErrorKind::*;

// 49
impl From<TryReserveErrorKind> for TryReserveError {
    #[inline(always)]
    fn from(kind: TryReserveErrorKind) -> Self {
        Self { kind }
    }
}

// 86
#[allow(dead_code)] // until we use Zeroed
enum AllocInit {
    /// The contents of the new memory are uninitialized.
    Uninitialized,
    /// The new memory is guaranteed to be zeroed.
    Zeroed,
}

/// A low-level utility for more ergonomically allocating, reallocating, and deallocating
/// a buffer of memory on the heap without having to worry about all the corner cases
/// involved. This type is excellent for building your own data structures like Vec and VecDeque.
/// In particular:
///
/// * Produces `NonNull::dangling()` on zero-sized types.
/// * Produces `NonNull::dangling()` on zero-length allocations.
/// * Avoids freeing `NonNull::dangling()`.
/// * Catches all overflows in capacity computations (promotes them to "capacity overflow" panics).
/// * Guards against 32-bit systems allocating more than isize::MAX bytes.
/// * Guards against overflowing your length.
/// * Calls `handle_alloc_error` for fallible allocations.
/// * Contains a `ptr::NonNull` and thus endows the user with all related benefits.
/// * Uses the excess returned from the allocator to use the largest available capacity.
///
/// This type does not in anyway inspect the memory that it manages. When dropped it *will*
/// free its memory, but it *won't* try to drop its contents. It is up to the user of `RawVec`
/// to handle the actual things *stored* inside of a `RawVec`.
///
/// Note that the excess of a zero-sized types is always infinite, so `capacity()` always returns
/// `usize::MAX`. This means that you need to be careful when round-tripping this type with a
/// `Box<[T]>`, since `capacity()` won't yield the length.
///
// 116
pub(crate) struct RawVec<T, A: Allocator = Global> {
    ptr: NonNull<T>,
    cap: usize,
    alloc: A,
}

// 140
impl<T> RawVec<T, Global> {
    /// Creates a `RawVec` (on the system heap) with exactly the
    /// capacity and alignment requirements for a `[T; capacity]`. This is
    /// equivalent to calling `RawVec::new` when `capacity` is `0` or `T` is
    /// zero-sized. Note that if `T` is zero-sized this means you will
    /// *not* get a `RawVec` with the requested capacity.
    ///
    #[must_use]
    #[inline(always)]
    // 167
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, Global)
    }

    /// Like `with_capacity`, but guarantees the buffer is zeroed.
    #[must_use]
    #[inline(always)]
    // 175
    pub fn with_capacity_zeroed(capacity: usize) -> Self {
        Self::with_capacity_zeroed_in(capacity, Global)
    }
}

// 180
impl<T, A: Allocator> RawVec<T, A> {
    /// Like `new`, but parameterized over the choice of allocator for
    /// the returned `RawVec`.
    #[inline(always)]
    // 197
    pub const fn new_in(alloc: A) -> Self {
        // `cap: 0` means "unallocated". zero-sized types are ignored.
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            alloc,
        }
    }

    /// Like `with_capacity`, but parameterized over the choice of
    /// allocator for the returned `RawVec`.
    #[inline(always)]
    // 210
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        Self::allocate_in(capacity, AllocInit::Uninitialized, alloc)
    }

    /// Like `with_capacity_zeroed`, but parameterized over the choice
    /// of allocator for the returned `RawVec`.
    #[inline(always)]
    // 218
    pub fn with_capacity_zeroed_in(capacity: usize, alloc: A) -> Self {
        Self::allocate_in(capacity, AllocInit::Zeroed, alloc)
    }

    /// Converts the entire buffer into `Box<[MaybeUninit<T>]>` with the specified `len`.
    ///
    /// Note that this will correctly reconstitute any `cap` changes
    /// that may have been performed. (See description of type for details.)
    ///
    #[inline(always)]
    // 235
    pub unsafe fn into_box(self, len: usize) -> Box<[MaybeUninit<T>], A> {
        // Sanity-check one half of the safety requirement (we cannot check the other half).
        #[cfg(not(feature = "defmt"))]
        debug_assert!(
            len <= self.capacity(),
            "`len` must be smaller than or equal to `self.capacity()`",
        );

        #[cfg(all(debug_assertions, feature = "defmt"))]
        if len > self.capacity() {
            panic!("`len` must be smaller than or equal to `self.capacity()`");
        }

        let me = ManuallyDrop::new(self);
        unsafe {
            let slice = slice::from_raw_parts_mut(me.ptr() as *mut MaybeUninit<T>, len);
            Box::<[MaybeUninit<T>], A>::from_raw_in(slice, ptr::read(&me.alloc))
        }
    }

    // 251
    #[inline(always)]
    fn allocate_in(capacity: usize, init: AllocInit, alloc: A) -> Self {
        // Don't allocate here because `Drop` will not deallocate when `capacity` is 0.
        if mem::size_of::<T>() == 0 || capacity == 0 {
            Self::new_in(alloc)
        } else {
            // We avoid `unwrap_or_else` here because it bloats the amount of
            // LLVM IR generated.
            let layout = match Layout::array::<T>(capacity) {
                Ok(layout) => layout,
                Err(_) => capacity_overflow(),
            };
            match alloc_guard(layout.size()) {
                Ok(_) => {}
                Err(_) => capacity_overflow(),
            }
            let result = match init {
                AllocInit::Uninitialized => alloc.allocate(layout),
                AllocInit::Zeroed => alloc.allocate_zeroed(layout),
            };
            let ptr = match result {
                Ok(ptr) => ptr,
                Err(_) => handle_alloc_error(layout),
            };

            // Allocators currently return a `NonNull<[u8]>` whose length
            // matches the size requested. If that ever changes, the capacity
            // here should change to `ptr.len() / mem::size_of::<T>()`.
            Self {
                ptr: unsafe { NonNull::new_unchecked(ptr.cast().as_ptr()) },
                cap: capacity,
                alloc,
            }
        }
    }

    /// Gets a raw pointer to the start of the allocation. Note that this is
    /// `NonNull::dangling()` if `capacity == 0` or `T` is zero-sized. In the former case, you must
    /// be careful.
    #[inline(always)]
    // 309
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Gets the capacity of the allocation.
    ///
    /// This will always be `usize::MAX` if `T` is zero-sized.
    #[inline(always)]
    // 317
    pub fn capacity(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.cap
        }
    }
}

#[inline(always)]
// 628
fn alloc_guard(alloc_size: usize) -> Result<(), TryReserveError> {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        Err(CapacityOverflow.into())
    } else {
        Ok(())
    }
}

// One central function responsible for reporting capacity overflows. This'll
// ensure that the code generation related to these panics is minimal as there's
// only one location which panics rather than a bunch throughout the module.
// 640
fn capacity_overflow() -> ! {
    panic!("capacity overflow");
}
