//! Random number generation traits
//!
//! This crate is mainly of interest to crates publishing implementations of
//! [`RngCore`]. Other users are encouraged to use the [`rand`] crate instead
//! which re-exports the main traits and error types.
//!
//! [`RngCore`] is the core trait implemented by algorithmic pseudo-random number
//! generators and external random-number sources.

#![no_std]

// 131
pub trait RngCore {}
