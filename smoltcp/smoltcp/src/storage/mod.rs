/*! Specialized containers.

The `storage` module provides containers for use in other modules.
The containers support both pre-allocated memory, without the `std`
or `alloc` crates being available, and heap-allocated memory.
*/

// 9
mod packet_buffer;
mod ring_buffer;

// 13
pub use self::packet_buffer::{PacketBuffer, PacketMetadata};
pub use self::ring_buffer::RingBuffer;

/// Error returned when enqueuing into a full buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 26
pub struct Full;

/// Error returned when dequeuing from an empty buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 31
pub struct Empty;
