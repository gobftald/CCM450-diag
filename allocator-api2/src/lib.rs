#![no_std]

#[allow(unused_imports)]
#[macro_use(panic)]
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
mod raw_vec;

#[cfg(feature = "alloc")]
// 33
mod unique;

// 108
#[cfg(feature = "alloc")]
#[inline(always)]
fn invalid_mut<T>(addr: usize) -> *mut T {
    //#[allow(clippy::useless_transmute, clippy::transmutes_expressible_as_ptr_casts)]
    unsafe { core::mem::transmute(addr) }
}
