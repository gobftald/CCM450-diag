use core::ptr::null_mut;
use core::ptr::NonNull;

use crate::{align_down_size, align_up};

/// A sorted list of holes. It uses the the holes itself to store its nodes.
//12
pub struct HoleList {
    pub(crate) first: Hole, // dummy
    pub(crate) bottom: *mut u8,
    pub(crate) top: *mut u8,
    pub(crate) pending_extend: u8,
}

/// A block containing free memory. It points to the next hole and thus forms a linked list.
// 26
pub(crate) struct Hole {
    pub size: usize,
    pub next: Option<NonNull<Hole>>,
}

// 258
impl HoleList {
    /// Creates an empty `HoleList`.
    // 260
    pub const fn empty() -> HoleList {
        HoleList {
            first: Hole {
                size: 0,
                next: None,
            },
            bottom: null_mut(),
            top: null_mut(),
            pending_extend: 0,
        }
    }

    // 329
    pub unsafe fn new(hole_addr: *mut u8, hole_size: usize) -> HoleList {
        assert_eq!(size_of::<Hole>(), Self::min_size());
        assert!(hole_size >= size_of::<Hole>());

        let aligned_hole_addr = align_up(hole_addr, align_of::<Hole>());
        let requested_hole_size = hole_size - ((aligned_hole_addr as usize) - (hole_addr as usize));
        let aligned_hole_size = align_down_size(requested_hole_size, align_of::<Hole>());

        let ptr = aligned_hole_addr as *mut Hole;
        ptr.write(Hole {
            size: aligned_hole_size,
            next: None,
        });

        /*
        assert_eq!(
            hole_addr.wrapping_add(hole_size),
            aligned_hole_addr.wrapping_add(requested_hole_size)
        );
        */

        HoleList {
            first: Hole {
                size: 0,
                next: Some(NonNull::new_unchecked(ptr)),
            },
            bottom: aligned_hole_addr,
            top: aligned_hole_addr.wrapping_add(aligned_hole_size),
            pending_extend: (requested_hole_size - aligned_hole_size) as u8,
        }
    }

    /// Returns the minimal allocation size. Smaller allocations or deallocations are not allowed.
    // 428
    pub fn min_size() -> usize {
        size_of::<usize>() * 2
    }
}
