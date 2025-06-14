#![no_std]

#[macro_use(panic)]
extern crate console;

use core::cell::Cell;

// 146
mod macros;

// 148
use core::alloc::{GlobalAlloc, Layout};

use core::ptr::null_mut;

// 157
use enumset::{EnumSet, EnumSetType};
use linked_list_allocator::Heap;

/// The global allocator instance
#[global_allocator]
// 162
pub static mut HEAP: EspHeap = EspHeap::empty();

// 164
const NON_REGION: Option<HeapRegion> = None;

//#[derive(EnumSetType, Debug)]
#[derive(EnumSetType)]
/// Describes the properties of a memory region
// 183
pub enum MemoryCapability {
    /// Memory must be internal; specifically it should not disappear when
    /// flash/spiram cache is switched off
    Internal,
    /// Memory must be in SPI RAM
    External,
}

/// A memory region to be used as heap memory
// 261
pub struct HeapRegion {
    heap: Heap,
    capabilities: EnumSet<MemoryCapability>,
}

// 266
impl HeapRegion {
    /// Create a new [HeapRegion] with the given capabilities
    ///
    /// # Safety
    ///
    /// - The supplied memory region must be available for the entire program
    ///   (`'static`).
    /// - The supplied memory region must be exclusively available to the heap
    ///   only, no aliasing.
    /// - `size > 0`.
    // 276
    pub unsafe fn new(
        heap_bottom: *mut u8,
        size: usize,
        capabilities: EnumSet<MemoryCapability>,
    ) -> Self {
        unsafe {
            let mut heap = Heap::empty();
            heap.init(heap_bottom, size);

            Self { heap, capabilities }
        }
    }
}

/// Internal stats to keep track across multiple regions.
#[cfg(feature = "internal-heap-stats")]
// 373
struct InternalHeapStats {
    max_usage: usize,
    total_allocated: usize,
    total_freed: usize,
}

/// A memory allocator
///
/// In addition to what Rust's memory allocator can do it allows to allocate
/// memory in regions satisfying specific needs.
// 383
pub struct EspHeap {
    //heap: Mutex<RefCell<[Option<HeapRegion>; 3]>>,
    heap: Cell<[Option<HeapRegion>; 3]>,
    #[cfg(feature = "internal-heap-stats")]
    //internal_heap_stats: Mutex<RefCell<InternalHeapStats>>,
    internal_heap_stats: InternalHeapStats,
}

// 389
impl EspHeap {
    /// Crate a new UNINITIALIZED heap allocator
    // 391
    pub const fn empty() -> Self {
        EspHeap {
            //heap: Mutex::new(RefCell::new([NON_REGION; 3])),
            heap: Cell::new([NON_REGION; 3]),
            #[cfg(feature = "internal-heap-stats")]
            //internal_heap_stats: Mutex::new(RefCell::new(InternalHeapStats {
            internal_heap_stats: InternalHeapStats {
                max_usage: 0,
                total_allocated: 0,
                total_freed: 0,
            },
        }
    }

    /// Add a memory region to the heap
    ///
    /// `heap_bottom` is a pointer to the location of the bottom of the heap.
    ///
    /// `size` is the size of the heap in bytes.
    ///
    /// You can add up to three regions per allocator.
    ///
    /// Note that:
    ///
    /// - Memory is allocated from the first suitable memory region first
    ///
    /// - The heap grows "upwards", towards larger addresses. Thus `end_addr`
    ///   must be larger than `start_addr`
    ///
    /// - The size of the heap is `(end_addr as usize) - (start_addr as usize)`.
    ///   The allocator won't use the byte at `end_addr`.
    ///
    /// # Safety
    ///
    /// - The supplied memory region must be available for the entire program (a
    ///   `'static` lifetime).
    /// - The supplied memory region must be exclusively available to the heap
    ///   only, no aliasing.
    /// - `size > 0`.
    // 428
    pub unsafe fn add_region(&mut self, region: HeapRegion) {
        critical_section::with(|_| {
            //let mut regions = self.heap.borrow_ref_mut(cs);
            let mut regions = self.heap.get_mut();
            let free = regions
                .iter()
                .enumerate()
                .find(|v| v.1.is_none())
                .map(|v| v.0);
            if let Some(free) = free {
                regions[free] = Some(region);
            } else {
                panic!(
                    "Exceeded the maximum of {} heap memory regions",
                    regions.len()
                );
            }
        });
    }
}

// 596
unsafe impl GlobalAlloc for EspHeap {
    // 597
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        null_mut()
    }

    // 601
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {}
}
