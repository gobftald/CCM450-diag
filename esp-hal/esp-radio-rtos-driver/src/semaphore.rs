//! Semaphores
//!
//! Semaphores are synchronization primitives that allow threads to coordinate their execution.
//! They are used to control access to a shared resource by limiting the number of threads that can
//! access it simultaneously.
//!
//! esp-radio sometimes mixes up semaphores and mutexes (FreeRTOS allows this), so this crate
//! exposes a single interface to work with both.
//!
//! ## Implementation
//!
//! Implement the `SemaphoreImplementation` trait for an object, and use the
//! `register_semaphore_implementation` to register that implementation for esp-radio.
//!
//! See the [`SemaphoreImplementation`] documentation for more information.
//!
//! ## Usage
//!
//! Users should use [`SemaphoreHandle`] to interact with semaphores created by the driver
//! implementation. Use [`SemaphoreKind`] to specify the type of semaphore or mutex to create.
//!
//! > Note that the only expected user of this crate is esp-radio.

// 24
use core::ptr::NonNull;

/// Pointer to an opaque semaphore created by the driver implementation.
// 27
pub type SemaphorePtr = NonNull<()>;
