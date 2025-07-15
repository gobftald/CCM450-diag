#![allow(dead_code)]

#[macro_use]
#[path = "gen/utils.rs"]
mod generated;

use core::sync::atomic::Ordering;

// 11
macro_rules! static_assert {
    ($cond:expr $(,)?) => {{
        let [()] = [(); (true /* type check */ & $cond) as usize];
    }};
}

// 17
macro_rules! static_assert_layout {
    ($atomic_type:ty, $value_type:ty) => {
        static_assert!(
            core::mem::align_of::<$atomic_type>() == core::mem::size_of::<$atomic_type>()
        );
        static_assert!(core::mem::size_of::<$atomic_type>() == core::mem::size_of::<$value_type>());
    };
}

// https://github.com/rust-lang/rust/blob/1.84.0/library/core/src/sync/atomic.rs#L3338
#[inline]
#[cfg_attr(
    all(debug_assertions, not(portable_atomic_no_track_caller)),
    track_caller
)]
// 284
pub(crate) fn assert_load_ordering(order: Ordering) {
    match order {
        Ordering::Acquire | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Release => panic!("there is no such thing as a release load"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release load"),
        _ => unreachable!(),
    }
}

// https://github.com/rust-lang/rust/blob/1.84.0/library/core/src/sync/atomic.rs#L3323
#[inline]
#[cfg_attr(
    all(debug_assertions, not(portable_atomic_no_track_caller)),
    track_caller
)]
// 295
pub(crate) fn assert_store_ordering(order: Ordering) {
    match order {
        Ordering::Release | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Acquire => panic!("there is no such thing as an acquire store"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release store"),
        _ => unreachable!(),
    }
}
