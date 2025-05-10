//! Interrupts

pub mod machine;

#[cfg(not(feature = "s-mode"))]
pub use machine::*;
