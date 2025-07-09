#![no_std]

#[allow(unused_imports)]
#[macro_use(panic, unreachable, debug_assert, debug_assert_eq)]
extern crate console;

// 9
extern crate alloc as alloc_crate;

// 14
pub mod alloc;

#[cfg(feature = "alloc")]
// 17
pub mod boxed;

#[cfg(feature = "alloc")]
// 20
//mod raw_vec;
pub mod raw_vec;

#[cfg(feature = "alloc")]
// 22
pub mod vec;

#[cfg(feature = "alloc")]
// 33
mod unique;

#[cfg(feature = "alloc")]
#[track_caller]
#[inline(always)]
#[cfg(debug_assertions)]
// 79
unsafe fn assume(v: bool) {
    if !v {
        //core::unreachable!()
        unreachable!()
    }
}

#[cfg(feature = "alloc")]
#[track_caller]
#[inline(always)]
#[cfg(not(debug_assertions))]
// 89
unsafe fn assume(v: bool) {
    if !v {
        unsafe {
            core::hint::unreachable_unchecked();
        }
    }
}

// 108
#[cfg(feature = "alloc")]
#[inline(always)]
fn invalid_mut<T>(addr: usize) -> *mut T {
    //#[allow(clippy::useless_transmute, clippy::transmutes_expressible_as_ptr_casts)]
    unsafe { core::mem::transmute(addr) }
}

pub mod linear_map;
