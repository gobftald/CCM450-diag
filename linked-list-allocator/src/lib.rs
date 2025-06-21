#![no_std]

#[macro_use(panic, assert, assert_eq)]
extern crate console;

// 13
use core::alloc::Layout;

// 19
use core::ptr::NonNull;

// 25
use hole::HoleList;

// 29
pub mod hole;

/// A fixed size heap backed by a linked list of free memory blocks.
// 34
pub struct Heap {
    used: usize,
    holes: HoleList,
}

// 55
impl Heap {
    /// Creates an empty heap. All allocate calls will return `None`.
    // 57
    pub const fn empty() -> Heap {
        Heap {
            used: 0,
            holes: HoleList::empty(),
        }
    }

    // 90
    pub unsafe fn init(&mut self, heap_bottom: *mut u8, heap_size: usize) {
        self.used = 0;
        self.holes = HoleList::new(heap_bottom, heap_size);
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `None`.
    /// This function scans the list of free memory blocks and uses the first block that is big
    /// enough. The runtime is in O(n) where n is the number of free blocks, but it should be
    /// reasonably fast for small allocations.
    //
    // NOTE: We could probably replace this with an `Option` instead of a `Result` in a later
    // release to remove this clippy warnings
    // 182
    pub fn allocate_first_fit(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        match self.holes.allocate_first_fit(layout) {
            Ok((ptr, aligned_layout)) => {
                self.used += aligned_layout.size();
                Ok(ptr)
            }
            Err(err) => Err(err),
        }
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate_first_fit` function with identical size and alignment.
    ///
    /// This function walks the list of free memory blocks and inserts the freed block at the
    /// correct place. If the freed block is adjacent to another free block, the blocks are merged
    /// again. This operation is in `O(n)` since the list needs to be sorted by address.
    ///
    /// # Safety
    ///
    /// `ptr` must be a pointer returned by a call to the [`allocate_first_fit`] function with
    /// identical layout. Undefined behavior may occur for invalid arguments.
    // 203
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.used -= self.holes.deallocate(ptr, layout).size();
    }

    /// Returns the bottom address of the heap.
    ///
    /// The bottom pointer is automatically aligned, so the returned pointer
    /// might be larger than the bottom pointer used for initialization.
    // 211
    pub fn bottom(&self) -> *mut u8 {
        self.holes.bottom
    }

    /// Returns the size of the heap.
    ///
    /// This is the size the heap is using for allocations, not necessarily the
    /// total amount of bytes given to the heap. To determine the exact memory
    /// boundaries, use [`bottom`][Self::bottom] and [`top`][Self::top].
    // 220
    pub fn size(&self) -> usize {
        unsafe { self.holes.top.offset_from(self.holes.bottom) as usize }
    }

    /// Return the top address of the heap.
    ///
    /// Note: The heap may choose to not use bytes at the end for allocations
    /// until there is enough room for metadata, but it still retains ownership
    /// over memory from [`bottom`][Self::bottom] to the address returned.
    // 229
    pub fn top(&self) -> *mut u8 {
        unsafe { self.holes.top.add(self.holes.pending_extend as usize) }
    }

    /// Returns the size of the used part of the heap
    // 234
    pub fn used(&self) -> usize {
        self.used
    }

    /// Returns the size of the free part of the heap
    // 239
    pub fn free(&self) -> usize {
        self.size() - self.used
    }
}

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
// 351
pub fn align_down_size(size: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        size & !(align - 1)
    } else if align == 0 {
        size
    } else {
        panic!("`align` must be a power of 2");
    }
}

// 358
pub fn align_up_size(size: usize, align: usize) -> usize {
    align_down_size(size + align - 1, align)
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
// 367
pub fn align_up(addr: *mut u8, align: usize) -> *mut u8 {
    let offset = addr.align_offset(align);
    addr.wrapping_add(offset)
}
