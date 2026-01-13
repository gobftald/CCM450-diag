#![no_std]

#[cfg(not(feature = "_generic-queue"))]
pub mod queue_integrated;

#[cfg(not(feature = "_generic-queue"))]
pub use queue_integrated::Queue;
