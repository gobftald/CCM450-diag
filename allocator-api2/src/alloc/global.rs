use core::ptr::NonNull;

// 3
use alloc_crate::alloc::{alloc, alloc_zeroed, dealloc};

// 5
use crate::invalid_mut;

// 7
use super::{AllocError, Allocator, Layout};

/// The global memory allocator.
///
/// This type implements the [`Allocator`] trait by forwarding calls
/// to the allocator registered with the `#[global_allocator]` attribute
/// if there is one, or the `std` crate’s default.
///
#[derive(Copy, Clone, Default, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 18
pub struct Global;

// 20
impl Global {
    #[inline(always)]
    fn alloc_impl(&self, layout: Layout, zeroed: bool) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(unsafe {
                NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                    invalid_mut(layout.align()),
                    0,
                ))
            }),
            // SAFETY: `layout` is non-zero in size,
            size => unsafe {
                let raw_ptr = if zeroed {
                    alloc_zeroed(layout)
                } else {
                    alloc(layout)
                };
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(NonNull::new_unchecked(core::ptr::slice_from_raw_parts_mut(
                    ptr.as_ptr(),
                    size,
                )))
            },
        }
    }
}

// 97
unsafe impl Allocator for Global {
    #[inline(always)]
    // 99
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl(layout, false)
    }

    #[inline(always)]
    // 104
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl(layout, true)
    }

    #[inline(always)]
    // 109
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            // SAFETY: `layout` is non-zero in size,
            // other conditions must be upheld by the caller
            unsafe { dealloc(ptr.as_ptr(), layout) }
        }
    }
}
