// 3
use crate::storage::{Full, RingBuffer};

// 5
use super::Empty;

/// Size and header of a packet.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 10
pub struct PacketMetadata<H> {
    size: usize,
    header: Option<H>,
}

// 15
impl<H> PacketMetadata<H> {
    // 22
    fn padding(size: usize) -> PacketMetadata<H> {
        PacketMetadata {
            size: size,
            header: None,
        }
    }

    // 29
    fn packet(size: usize, header: H) -> PacketMetadata<H> {
        PacketMetadata {
            size: size,
            header: Some(header),
        }
    }

    // 36
    fn is_padding(&self) -> bool {
        self.header.is_none()
    }
}

/// An UDP packet ring buffer.
#[derive(Debug)]
// 43
pub struct PacketBuffer<'a, H: 'a> {
    metadata_ring: RingBuffer<'a, PacketMetadata<H>>,
    payload_ring: RingBuffer<'a, u8>,
}

// 48
impl<'a, H> PacketBuffer<'a, H> {
    /// Query whether the buffer is empty.
    // 65
    pub fn is_empty(&self) -> bool {
        self.metadata_ring.is_empty()
    }

    // There is currently no enqueue_with() because of the complexity of managing padding
    // in case of failure.

    /// Enqueue a single packet with the given header into the buffer, and
    /// return a reference to its payload, or return `Err(Full)`
    /// if the buffer is full.
    // 80
    pub fn enqueue(&mut self, size: usize, header: H) -> Result<&mut [u8], Full> {
        if self.payload_ring.capacity() < size || self.metadata_ring.is_full() {
            return Err(Full);
        }

        // Ring is currently empty.  Clear it (resetting `read_at`) to maximize
        // for contiguous space.
        if self.payload_ring.is_empty() {
            self.payload_ring.clear();
        }

        let window = self.payload_ring.window();
        let contig_window = self.payload_ring.contiguous_window();

        if window < size {
            return Err(Full);
        } else if contig_window < size {
            if window - contig_window < size {
                // The buffer length is larger than the current contiguous window
                // and is larger than the contiguous window will be after adding
                // the padding necessary to circle around to the beginning of the
                // ring buffer.
                return Err(Full);
            } else {
                // Add padding to the end of the ring buffer so that the
                // contiguous window is at the beginning of the ring buffer.
                *self.metadata_ring.enqueue_one()? = PacketMetadata::padding(contig_window);
                // note(discard): function does not write to the result
                // enqueued padding buffer location
                let _buf_enqueued = self.payload_ring.enqueue_many(contig_window);
            }
        }

        *self.metadata_ring.enqueue_one()? = PacketMetadata::packet(size, header);

        let payload_buf = self.payload_ring.enqueue_many(size);
        debug_assert!(payload_buf.len() == size);
        Ok(payload_buf)
    }

    // 166
    fn dequeue_padding(&mut self) {
        let _ = self.metadata_ring.dequeue_one_with(|metadata| {
            if metadata.is_padding() {
                // note(discard): function does not use value of dequeued padding bytes
                let _buf_dequeued = self.payload_ring.dequeue_many(metadata.size);
                Ok(()) // dequeue metadata
            } else {
                Err(()) // don't dequeue metadata
            }
        });
    }

    /// Call `f` with a single packet from the buffer, and dequeue the packet if `f`
    /// returns successfully, or return `Err(EmptyError)` if the buffer is empty.
    // 180
    pub fn dequeue_with<'c, R, E, F>(&'c mut self, f: F) -> Result<Result<R, E>, Empty>
    where
        F: FnOnce(&mut H, &'c mut [u8]) -> Result<R, E>,
    {
        self.dequeue_padding();

        self.metadata_ring.dequeue_one_with(|metadata| {
            self.payload_ring
                .dequeue_many_with(|payload_buf| {
                    debug_assert!(payload_buf.len() >= metadata.size);

                    match f(
                        //metadata.header.as_mut().unwrap(),
                        unwrap!(metadata.header.as_mut()),
                        &mut payload_buf[..metadata.size],
                    ) {
                        Ok(val) => (metadata.size, Ok(val)),
                        Err(err) => (0, Err(err)),
                    }
                })
                .1
        })
    }
}
