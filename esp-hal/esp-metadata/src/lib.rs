//! Metadata for Espressif devices, primarily intended for use in build scripts.
#![cfg_attr(not(feature = "build"), no_std)]

#[cfg(feature = "build")]
mod generate_cfg;

#[cfg(feature = "build")]
pub use generate_cfg::*;
