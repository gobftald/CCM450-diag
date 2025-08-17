// 5
use core::cmp;
use managed::ManagedSlice;

// 10
use super::{Empty, Full};

/// A ring buffer.
///
/// This ring buffer implementation provides many ways to interact with it:
///
///   * Enqueueing or dequeueing one element from corresponding side of the buffer;
///   * Enqueueing or dequeueing a slice of elements from corresponding side of the buffer;
///   * Accessing allocated and unallocated areas directly.
///
/// It is also zero-copy; all methods provide references into the buffer's storage.
/// Note that all references are mutable; it is considered more important to allow
/// in-place processing than to protect from accidental mutation.
///
/// This implementation is suitable for both simple uses such as a FIFO queue
/// of UDP packets, and advanced ones such as a TCP reassembly buffer.
#[derive(Debug)]
// 27
pub struct RingBuffer<'a, T: 'a> {
    storage: ManagedSlice<'a, T>,
    read_at: usize,
    length: usize,
}

// 33
impl<'a, T: 'a> RingBuffer<'a, T> {
    /// Create a ring buffer with the given storage.
    ///
    /// During creation, every element in `storage` is reset.
    // 37
    pub fn new<S>(storage: S) -> RingBuffer<'a, T>
    where
        S: Into<ManagedSlice<'a, T>>,
    {
        RingBuffer {
            storage: storage.into(),
            read_at: 0,
            length: 0,
        }
    }

    /// Clear the ring buffer.
    // 49
    pub fn clear(&mut self) {
        self.read_at = 0;
        self.length = 0;
    }

    /// Return the maximum number of elements in the ring buffer.
    // 55
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Return the current number of elements in the ring buffer.
    // 71
    pub fn len(&self) -> usize {
        self.length
    }

    /// Return the number of elements that can be added to the ring buffer.
    // 76
    pub fn window(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Return the largest number of elements that can be added to the buffer
    /// without wrapping around (i.e. in a single `enqueue_many` call).
    // 82
    pub fn contiguous_window(&self) -> usize {
        cmp::min(self.window(), self.capacity() - self.get_idx(self.length))
    }

    /// Query whether the buffer is empty.
    // 87
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Query whether the buffer is full.
    // 92
    pub fn is_full(&self) -> bool {
        self.window() == 0
    }

    /// Shorthand for `(self.read + idx) % self.capacity()` with an
    /// additional check to ensure that the capacity is not zero.
    // 98
    fn get_idx(&self, idx: usize) -> usize {
        let len = self.capacity();
        if len > 0 {
            (self.read_at + idx) % len
        } else {
            0
        }
    }

    /// Shorthand for `(self.read + idx) % self.capacity()` with no
    /// additional checks to ensure the capacity is not zero.
    // 109
    fn get_idx_unchecked(&self, idx: usize) -> usize {
        (self.read_at + idx) % self.capacity()
    }
}

/// This is the "discrete" ring buffer interface: it operates with single elements,
/// and boundary conditions (empty/full) are errors.
// 116
impl<'a, T: 'a> RingBuffer<'a, T> {
    /// Call `f` with a single buffer element, and enqueue the element if `f`
    /// returns successfully, or return `Err(Full)` if the buffer is full.
    // 119
    pub fn enqueue_one_with<'b, R, E, F>(&'b mut self, f: F) -> Result<Result<R, E>, Full>
    where
        F: FnOnce(&'b mut T) -> Result<R, E>,
    {
        if self.is_full() {
            return Err(Full);
        }

        let index = self.get_idx_unchecked(self.length);
        let res = f(&mut self.storage[index]);
        if res.is_ok() {
            self.length += 1;
        }
        Ok(res)
    }

    /// Enqueue a single element into the buffer, and return a reference to it,
    /// or return `Err(Full)` if the buffer is full.
    ///
    /// This function is a shortcut for `ring_buf.enqueue_one_with(Ok)`.
    // 139
    pub fn enqueue_one(&mut self) -> Result<&mut T, Full> {
        self.enqueue_one_with(Ok)?
    }

    /// Call `f` with a single buffer element, and dequeue the element if `f`
    /// returns successfully, or return `Err(Empty)` if the buffer is empty.
    // 145
    pub fn dequeue_one_with<'b, R, E, F>(&'b mut self, f: F) -> Result<Result<R, E>, Empty>
    where
        F: FnOnce(&'b mut T) -> Result<R, E>,
    {
        if self.is_empty() {
            return Err(Empty);
        }

        let next_at = self.get_idx_unchecked(1);
        let res = f(&mut self.storage[self.read_at]);

        if res.is_ok() {
            self.length -= 1;
            self.read_at = next_at;
        }
        Ok(res)
    }

    /// Dequeue an element from the buffer, and return a reference to it,
    /// or return `Err(Empty)` if the buffer is empty.
    ///
    /// This function is a shortcut for `ring_buf.dequeue_one_with(Ok)`.
    // 167
    pub fn dequeue_one(&mut self) -> Result<&mut T, Empty> {
        self.dequeue_one_with(Ok)?
    }
}

/// This is the "continuous" ring buffer interface: it operates with element slices,
/// and boundary conditions (empty/full) simply result in empty slices.
// 174
impl<'a, T: 'a> RingBuffer<'a, T> {
    /// Call `f` with the largest contiguous slice of unallocated buffer elements,
    /// and enqueue the amount of elements returned by `f`.
    ///
    /// # Panics
    /// This function panics if the amount of elements returned by `f` is larger
    /// than the size of the slice passed into it.
    // 181
    pub fn enqueue_many_with<'b, R, F>(&'b mut self, f: F) -> (usize, R)
    where
        F: FnOnce(&'b mut [T]) -> (usize, R),
    {
        if self.length == 0 {
            // Ring is currently empty. Reset `read_at` to optimize
            // for contiguous space.
            self.read_at = 0;
        }

        let write_at = self.get_idx(self.length);
        let max_size = self.contiguous_window();
        let (size, result) = f(&mut self.storage[write_at..write_at + max_size]);
        assert!(size <= max_size);
        self.length += size;
        (size, result)
    }

    /// Enqueue a slice of elements up to the given size into the buffer,
    /// and return a reference to them.
    ///
    /// This function may return a slice smaller than the given size
    /// if the free space in the buffer is not contiguous.
    #[must_use]
    // 205
    pub fn enqueue_many(&mut self, size: usize) -> &mut [T] {
        self.enqueue_many_with(|buf| {
            let size = cmp::min(size, buf.len());
            (size, &mut buf[..size])
        })
        .1
    }

    /// Call `f` with the largest contiguous slice of allocated buffer elements,
    /// and dequeue the amount of elements returned by `f`.
    ///
    /// # Panics
    /// This function panics if the amount of elements returned by `f` is larger
    /// than the size of the slice passed into it.
    // 239
    pub fn dequeue_many_with<'b, R, F>(&'b mut self, f: F) -> (usize, R)
    where
        F: FnOnce(&'b mut [T]) -> (usize, R),
    {
        let capacity = self.capacity();
        let max_size = cmp::min(self.len(), capacity - self.read_at);
        let (size, result) = f(&mut self.storage[self.read_at..self.read_at + max_size]);
        assert!(size <= max_size);
        self.read_at = if capacity > 0 {
            (self.read_at + size) % capacity
        } else {
            0
        };
        self.length -= size;
        (size, result)
    }

    /// Dequeue a slice of elements up to the given size from the buffer,
    /// and return a reference to them.
    ///
    /// This function may return a slice smaller than the given size
    /// if the allocated space in the buffer is not contiguous.
    #[must_use]
    // 262
    pub fn dequeue_many(&mut self, size: usize) -> &mut [T] {
        self.dequeue_many_with(|buf| {
            let size = cmp::min(size, buf.len());
            (size, &mut buf[..size])
        })
        .1
    }
}
