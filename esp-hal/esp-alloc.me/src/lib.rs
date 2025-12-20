#![no_std]
#![allow(static_mut_refs)]

// 146
mod macros;

// 148
use core::{
    alloc::{GlobalAlloc, Layout},
    cell::RefCell,
    fmt::Display,
    ptr::NonNull,
};

// 158
use linked_list_allocator::Heap;

/// The global allocator instance
#[global_allocator]
// 162
pub static mut HEAP: EspHeap = EspHeap::empty();

/// Stats for a heap allocator
///
// 305
#[derive(Debug)]
pub struct HeapStats {
    /// Total size of all combined heap regions in bytes.
    size: usize,

    /// Current usage of the heap across all configured regions in bytes.
    current_usage: usize,

    /// Estimation of the max used heap in bytes.
    max_usage: usize,

    /// Estimation of the total allocated bytes since initialization.
    total_allocated: usize,

    /// Estimation of the total freed bytes since initialization.
    total_freed: usize,
}

// 328
impl Display for HeapStats {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "HEAP INFO")?;
        writeln!(f, "Size: {}", self.size)?;
        writeln!(f, "Current usage: {}", self.current_usage)?;
        //#[cfg(feature = "internal-heap-stats")]
        //{
        writeln!(f, "Max usage: {}", self.max_usage)?;
        writeln!(f, "Total freed: {}", self.total_freed)?;
        writeln!(f, "Total allocated: {}", self.total_allocated)?;
        //}
        /*
        writeln!(f, "Memory Layout: ")?;
        for region in self.region_stats.iter() {
            if let Some(region) = region.as_ref() {
                region.fmt(f)?;
                writeln!(f)?;
            }
        }
        */
        Ok(())
    }
}

// 351
#[cfg(feature = "defmt")]
impl defmt::Format for HeapStats {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(fmt, "HEAP INFO\n");
        defmt::write!(fmt, "Size: {}\n", self.size);
        defmt::write!(fmt, "Current usage: {}\n", self.current_usage);
        //#[cfg(feature = "internal-heap-stats")]
        //{
        defmt::write!(fmt, "Max usage: {}\n", self.max_usage);
        defmt::write!(fmt, "Total freed: {}\n", self.total_freed);
        defmt::write!(fmt, "Total allocated: {}\n", self.total_allocated);
        //}
        /*
        defmt::write!(fmt, "Memory Layout:\n");
        for region in self.region_stats.iter() {
            if let Some(region) = region.as_ref() {
                defmt::write!(fmt, "{}\n", region);
            }
        }
        */
    }
}

/// Internal stats to keep track across multiple regions.
#[cfg(feature = "internal-heap-stats")]
// 373
//struct InternalHeapStats {
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
    heap: RefCell<Option<Heap>>,
    #[cfg(feature = "internal-heap-stats")]
    //internal_heap_stats: Mutex<RefCell<InternalHeapStats>>,
    internal_heap_stats: RefCell<InternalHeapStats>,
}

// 389
impl EspHeap {
    /// Crate a new UNINITIALIZED heap allocator
    // 391
    pub const fn empty() -> Self {
        EspHeap {
            //heap: Mutex::new(RefCell::new([NON_REGION; 3])),
            heap: RefCell::new(None),
            #[cfg(feature = "internal-heap-stats")]
            //internal_heap_stats: Mutex::new(RefCell::new(InternalHeapStats {
            internal_heap_stats: RefCell::new(InternalHeapStats {
                max_usage: 0,
                total_allocated: 0,
                total_freed: 0,
            }),
        }
    }

    // comes from skipped HeapRegion
    pub unsafe fn init(&self, heap_bottom: *mut u8, size: usize) {
        unsafe {
            let mut heap = Heap::empty();
            heap.init(heap_bottom, size);

            critical_section::with(|_| *HEAP.heap.borrow_mut() = Some(heap));
        }
    }

    /// Returns an estimate of the amount of bytes in use in all memory regions.
    // 449
    pub fn used(&self) -> usize {
        critical_section::with(|_| {
            //let regions = self.heap.borrow_ref(cs);
            /*
            for region in regions.iter() {
                if let Some(region) = region.as_ref() {
                    used += region.heap.used();
                }
            }
            */

            if let Some(heap) = self.heap.borrow().as_ref() {
                heap.used()
            } else {
                0
            }
        })
    }

    /// Return usage stats for the Heap.
    ///
    #[cfg(feature = "internal-heap-stats")]
    // 468
    pub fn stats(&self) -> HeapStats {
        //const EMPTY_REGION_STAT: Option<RegionStats> = None;
        //let mut region_stats: [Option<RegionStats>; 3] = [EMPTY_REGION_STAT; 3];

        critical_section::with(|_cs| {
            /*
            let mut used = 0;
            let mut free = 0;
            let regions = self.heap.borrow_ref(cs);
            for (id, region) in regions.iter().enumerate() {
                if let Some(region) = region.as_ref() {
                    let stats = region.stats();
                    free += stats.free;
                    used += stats.used;
                    region_stats[id] = Some(region.stats());
                }
            }

            cfg_if::cfg_if! {
                if #[cfg(feature = "internal-heap-stats")] {
                    let internal_heap_stats = self.internal_heap_stats.borrow_ref(cs);
                    HeapStats {
                        region_stats,
                        size: free + used,
                        current_usage: used,
                        max_usage: internal_heap_stats.max_usage,
                        total_allocated: internal_heap_stats.total_allocated,
                        total_freed: internal_heap_stats.total_freed,
                    }
                } else {
                    HeapStats {
                        region_stats,
                        size: free + used,
                        current_usage: used,
                    }
                }
            }
            */

            let mut used = 0;
            let mut free = 0;
            if let Some(heap) = self.heap.borrow().as_ref() {
                used = heap.used();
                free = heap.free();
            }

            let internal_heap_stats = self.internal_heap_stats.borrow();
            HeapStats {
                size: free + used,
                current_usage: used,
                max_usage: internal_heap_stats.max_usage,
                total_allocated: internal_heap_stats.total_allocated,
                total_freed: internal_heap_stats.total_freed,
            }
        })
    }
}

unsafe impl GlobalAlloc for EspHeap {
    /// Allocate memory
    // 597
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //unsafe { self.alloc_caps(EnumSet::empty(), layout) }

        critical_section::with(|_| {
            #[cfg(feature = "internal-heap-stats")]
            let before = self.used();

            let res = if let Some(heap) = self.heap.borrow_mut().as_mut() {
                let res = heap.allocate_first_fit(layout);
                if let Ok(res) = res {
                    res
                } else {
                    return core::ptr::null_mut();
                }
            } else {
                return core::ptr::null_mut();
            };

            #[cfg(feature = "internal-heap-stats")]
            {
                let mut internal_heap_stats = self.internal_heap_stats.borrow_mut();
                let used = self.used();

                internal_heap_stats.total_allocated += used - before;
                internal_heap_stats.max_usage = core::cmp::max(internal_heap_stats.max_usage, used);
            }

            res.as_ptr()
        })
    }

    // 601
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            if ptr.is_null() {
                return;
            }

            critical_section::with(|_| {
                #[cfg(feature = "internal-heap-stats")]
                let before = self.used();

                if let Some(heap) = self.heap.borrow_mut().as_mut() {
                    if heap.bottom() <= ptr && heap.top() >= ptr {
                        heap.deallocate(NonNull::new_unchecked(ptr), layout);
                    }
                }

                #[cfg(feature = "internal-heap-stats")]
                {
                    let mut internal_heap_stats = self.internal_heap_stats.borrow_mut();

                    internal_heap_stats.total_freed += before - self.used();
                }
            })
        }
    }
}
