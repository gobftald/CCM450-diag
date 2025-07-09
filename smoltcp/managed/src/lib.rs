#![no_std]

//! A library that provides a way to logically own objects, whether or not
//! heap allocation is available.

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

mod slice;

pub use slice::ManagedSlice;
