#![no_std]

#[macro_use(panic, assert, assert_eq)]
extern crate console;

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

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
// 367
pub fn align_up(addr: *mut u8, align: usize) -> *mut u8 {
    let offset = addr.align_offset(align);
    addr.wrapping_add(offset)
}
