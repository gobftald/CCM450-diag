//! An attribute to create an atomic wrapper around a C-style enum.
//!

#![no_std]

#[cfg(not(feature = "portable-atomic"))]
pub use core::sync::atomic;

#[cfg(feature = "portable-atomic")]
pub use portable_atomic as atomic;

pub use portable_atomic_enum_macros::atomic_enum;
