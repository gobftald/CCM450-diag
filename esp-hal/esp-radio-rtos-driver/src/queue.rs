// In the esp-radio-rtos-driver crate, the CompatQueue is a specialized, internal implementation designed
// to bridge the gap between the high-level Rust async ecosystem and the low-level expectations of the radio
// driver, whereas the esp-rtos queue is a general-purpose RTOS primitive.
// The primary differences are:
// 1. Purpose: Abstraction vs. Implementation
// - CompatQueue (Local): This is a "Compatibility Layer" (hence the name). Its main role is to wrap an
//   existing synchronization primitive to satisfy a specific interface required by the radio hardware's
//   internal tasks. It is often a thin wrapper that "tricks" the driver into thinking it has a standard
//   queue while actually using an underlying structure like a WaitQueue or an embassy-sync primitive.
// - esp-rtos Queue: This is a full implementation of a thread-safe, blocking/async queue. It manages its own
//   internal buffers, storage, and task waiting lists directly.
// 2. Context-Aware Synchronization
// - CompatQueue: It is specifically designed to handle ISR-to-Task transitions more efficiently for radio
//   packets. It often bypasses the standard RTOS "tick" timer to ensure that when a radio interrupt occurs,
//   the CompatQueue can wake a task with lower latency than a general-purpose queue.
// - esp-rtos Queue: Follows standard RTOS scheduling rules. While it supports ISR operations (via
//   try_take_from_isr), it is built for broader reliability across any application task rather than being
//   tuned for high-speed radio packet throughput.
// 3. Dependency Management
// - CompatQueue: Exists locally to decouple the radio driver from a specific RTOS version. This allows
//   esp-radio to run on different platforms (like ArielOS or bare-metal esp-hal) by simply swapping the
//   internal logic of the CompatQueue without changing the radio driver's core code.
// - esp-rtos Queue: Bound specifically to the esp-rtos runtime and the esp-hal hardware abstraction layer.
// 4. Memory Footprint
// - CompatQueue: Often uses zero-allocation or static allocation strategies tailored for the radio's
//   fixed-size command/event buffers.
// - esp-rtos Queue: Generally more flexible, allowing dynamic sizes but requiring more management overhead
//   (like tracking head/tail pointers and buffer limits) which can use more RAM per instance.
// Summary: Use the esp-rtos queue for your own application logic and inter-task communication. The
// CompatQueue is an internal detail you should only interact with if you are modifying how the radio driver
// itself manages its internal work-loops.

// 21
use core::{cell::UnsafeCell, ptr::NonNull};

/// Pointer to an opaque queue created by the driver implementation.
// 24
pub type QueuePtr = NonNull<()>;

// 118
pub trait QueueImplementation {
    /// Creates a new, empty queue instance.
    ///
    /// The queue must have a capacity for `capacity` number of `item_size` byte items.
    fn create(capacity: usize, item_size: usize) -> QueuePtr;

    /// Deletes a queue instance.
    ///
    /// # Safety
    ///
    /// `queue` must be a pointer returned from [`Self::create`].
    unsafe fn delete(queue: QueuePtr);

    /// Enqueues a high-priority item.
    ///
    /// If the queue is full, this function will block for the given timeout. If timeout is None,
    /// the function will block indefinitely.
    ///
    /// This function returns `true` if the item was successfully enqueued, `false` otherwise.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` can be dereferenced and points to an allocation of
    /// a size equal to the queue's item size.
    unsafe fn send_to_front(queue: QueuePtr, item: *const u8, timeout_us: Option<u32>) -> bool;

    /// Enqueues an item.
    ///
    /// If the queue is full, this function will block for the given timeout. If timeout is None,
    /// the function will block indefinitely.
    ///
    /// This function returns `true` if the item was successfully enqueued, `false` otherwise.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` can be dereferenced and points to an allocation of
    /// a size equal to the queue's item size.
    unsafe fn send_to_back(queue: QueuePtr, item: *const u8, timeout_us: Option<u32>) -> bool;

    /// Attempts to enqueues an item.
    ///
    /// If the queue is full, this function will immediately return `false`.
    ///
    /// The `higher_prio_task_waken` parameter is an optional mutable reference to a boolean flag.
    /// If the flag is `Some`, the implementation may set it to `true` to request a context switch.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` can be dereferenced and points to an allocation of
    /// a size equal to the queue's item size.
    unsafe fn try_send_to_back_from_isr(
        queue: QueuePtr,
        item: *const u8,
        higher_prio_task_waken: Option<&mut bool>,
    ) -> bool;

    /// Dequeues an item from the queue.
    ///
    /// If the queue is empty, this function will block for the given timeout. If timeout is None,
    /// the function will block indefinitely.
    ///
    /// This function returns `true` if the item was successfully dequeued, `false` otherwise.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` can be dereferenced and points to an allocation of
    /// a size equal to the queue's item size.
    unsafe fn receive(queue: QueuePtr, item: *mut u8, timeout_us: Option<u32>) -> bool;

    /// Attempts to dequeue an item from the queue.
    ///
    /// If the queue is empty, this function will return `false` immediately.
    ///
    /// The `higher_prio_task_waken` parameter is an optional mutable reference to a boolean flag.
    /// If the flag is `Some`, the implementation may set it to `true` to request a context switch.
    ///
    /// This function returns `true` if the item was successfully dequeued, `false` otherwise.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` can be dereferenced and points to an allocation of
    /// a size equal to the queue's item size.
    unsafe fn try_receive_from_isr(
        queue: QueuePtr,
        item: *mut u8,
        higher_prio_task_waken: Option<&mut bool>,
    ) -> bool;

    /// Removes an item from the queue.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `item` can be dereferenced and points to an allocation of
    /// a size equal to the queue's item size.
    unsafe fn remove(queue: QueuePtr, item: *const u8);

    /// Returns the number of messages in the queue.
    fn messages_waiting(queue: QueuePtr) -> usize;
}

// 218
#[macro_export]
macro_rules! register_queue_implementation {
    ($t: ty) => {
        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_create(capacity: usize, item_size: usize) -> $crate::queue::QueuePtr {
            <$t as $crate::queue::QueueImplementation>::create(capacity, item_size)
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_delete(queue: $crate::queue::QueuePtr) {
            unsafe { <$t as $crate::queue::QueueImplementation>::delete(queue) }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_send_to_front(
            queue: QueuePtr,
            item: *const u8,
            timeout_us: Option<u32>,
        ) -> bool {
            unsafe {
                <$t as $crate::queue::QueueImplementation>::send_to_front(queue, item, timeout_us)
            }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_send_to_back(
            queue: QueuePtr,
            item: *const u8,
            timeout_us: Option<u32>,
        ) -> bool {
            unsafe {
                <$t as $crate::queue::QueueImplementation>::send_to_back(queue, item, timeout_us)
            }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_try_send_to_back_from_isr(
            queue: QueuePtr,
            item: *const u8,
            higher_prio_task_waken: Option<&mut bool>,
        ) -> bool {
            unsafe {
                <$t as $crate::queue::QueueImplementation>::try_send_to_back_from_isr(
                    queue,
                    item,
                    higher_prio_task_waken,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_receive(queue: QueuePtr, item: *mut u8, timeout_us: Option<u32>) -> bool {
            unsafe { <$t as $crate::queue::QueueImplementation>::receive(queue, item, timeout_us) }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_try_receive_from_isr(
            queue: QueuePtr,
            item: *mut u8,
            higher_prio_task_waken: Option<&mut bool>,
        ) -> bool {
            unsafe {
                <$t as $crate::queue::QueueImplementation>::try_receive_from_isr(
                    queue,
                    item,
                    higher_prio_task_waken,
                )
            }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_remove(queue: QueuePtr, item: *mut u8) {
            unsafe { <$t as $crate::queue::QueueImplementation>::remove(queue, item) }
        }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_queue_messages_waiting(queue: QueuePtr) -> usize {
            unsafe { <$t as $crate::queue::QueueImplementation>::messages_waiting(queue) }
        }
    };
}

// 470
use alloc::{boxed::Box, vec};

// 472
use crate::semaphore::{SemaphoreHandle, SemaphoreKind};

// 474
struct QueueInner {
    storage: Box<[u8]>,
    item_size: usize,
    capacity: usize,
    count: usize,
    current_read: usize,
    current_write: usize,
}

// 483
impl QueueInner {
    fn get(&self, index: usize) -> &[u8] {
        let item_start = self.item_size * index;
        &self.storage[item_start..][..self.item_size]
    }

    // 489
    fn get_mut(&mut self, index: usize) -> &mut [u8] {
        let item_start = self.item_size * index;
        &mut self.storage[item_start..][..self.item_size]
    }

    // 494
    fn len(&self) -> usize {
        self.count
    }

    // 498
    fn send_to_back(&mut self, item: *const u8) {
        let item = unsafe { core::slice::from_raw_parts(item, self.item_size) };

        let dst = self.get_mut(self.current_write);
        dst.copy_from_slice(item);

        self.current_write = (self.current_write + 1) % self.capacity;
        self.count += 1;
    }

    // 508
    fn send_to_front(&mut self, item: *const u8) {
        let item = unsafe { core::slice::from_raw_parts(item, self.item_size) };

        self.current_read = (self.current_read + self.capacity - 1) % self.capacity;

        let dst = self.get_mut(self.current_read);
        dst.copy_from_slice(item);

        self.count += 1;
    }

    // 519
    fn read_from_front(&mut self, dst: *mut u8) {
        let dst = unsafe { core::slice::from_raw_parts_mut(dst, self.item_size) };

        let src = self.get(self.current_read);
        dst.copy_from_slice(src);

        self.current_read = (self.current_read + 1) % self.capacity;
        self.count -= 1;
    }

    // 529
    fn remove(&mut self, item: *const u8) -> bool {
        let count = self.len();

        if count == 0 {
            return false;
        }

        let mut tmp_item = vec![0; self.item_size];

        let mut found = false;
        let item_slice = unsafe { core::slice::from_raw_parts(item, self.item_size) };
        for _ in 0..count {
            self.read_from_front(tmp_item.as_mut_ptr().cast());

            if found || &tmp_item[..] != item_slice {
                self.send_to_back(tmp_item.as_mut_ptr().cast());
            } else {
                found = true;
            }

            // Note that even if we find our item, we'll need to keep cycling through everything to
            // keep insertion order.
        }

        found
    }
}

/// A suitable queue implementation that only requires semaphores from the OS.
///
/// Register in your OS implementation by adding the following code:
///
/// ```rust
/// use esp_radio_rtos_driver::{queue::CompatQueue, register_queue_implementation};
///
/// register_queue_implementation!(CompatQueue);
/// ```
// 566
pub struct CompatQueue {
    /// Allows interior mutability for the queue's inner state, when the mutex is held.
    inner: UnsafeCell<QueueInner>,

    semaphore_empty: SemaphoreHandle,
    semaphore_full: SemaphoreHandle,
    mutex: SemaphoreHandle,
}

// 575
impl CompatQueue {
    fn new(capacity: usize, item_size: usize) -> Self {
        let storage = vec![0; capacity * item_size].into_boxed_slice();
        let semaphore_empty = SemaphoreHandle::new(SemaphoreKind::Counting {
            max: capacity as u32,
            initial: capacity as u32,
        });
        let semaphore_full = SemaphoreHandle::new(SemaphoreKind::Counting {
            max: capacity as u32,
            initial: 0,
        });
        let mutex = SemaphoreHandle::new(SemaphoreKind::Mutex);
        Self {
            inner: UnsafeCell::new(QueueInner {
                storage,
                item_size,
                capacity,
                count: 0,
                current_read: 0,
                current_write: 0,
            }),
            semaphore_empty,
            semaphore_full,
            mutex,
        }
    }

    // 602
    unsafe fn from_ptr<'a>(ptr: QueuePtr) -> &'a Self {
        unsafe { ptr.cast::<Self>().as_ref() }
    }
}

// 607
impl QueueImplementation for CompatQueue {
    fn create(capacity: usize, item_size: usize) -> QueuePtr {
        let q = Box::new(CompatQueue::new(capacity, item_size));
        NonNull::from(Box::leak(q)).cast()
    }

    // 613
    unsafe fn delete(queue: QueuePtr) {
        let q = unsafe { Box::from_raw(queue.cast::<CompatQueue>().as_ptr()) };
        core::mem::drop(q);
    }

    // 618
    unsafe fn send_to_front(queue: QueuePtr, item: *const u8, timeout_us: Option<u32>) -> bool {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        if queue.semaphore_empty.take(timeout_us) {
            // The inner mutex shouldn't be held for a long time, but we still shouldn't block
            // indefinitely.
            if queue.mutex.take(timeout_us) {
                let inner = unsafe { &mut *queue.inner.get() };
                inner.send_to_front(item);

                queue.mutex.give();
                queue.semaphore_full.give();
                true
            } else {
                queue.semaphore_empty.give();
                false
            }
        } else {
            false
        }
    }

    // 640
    unsafe fn send_to_back(queue: QueuePtr, item: *const u8, timeout_us: Option<u32>) -> bool {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        if queue.semaphore_empty.take(timeout_us) {
            // The inner mutex shouldn't be held for a long time, but we still shouldn't block
            // indefinitely.
            if queue.mutex.take(timeout_us) {
                let inner = unsafe { &mut *queue.inner.get() };
                inner.send_to_back(item);

                queue.mutex.give();
                queue.semaphore_full.give();
                true
            } else {
                queue.semaphore_empty.give();
                false
            }
        } else {
            false
        }
    }

    // 662
    unsafe fn try_send_to_back_from_isr(
        queue: QueuePtr,
        item: *const u8,
        mut higher_prio_task_waken: Option<&mut bool>,
    ) -> bool {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        if queue
            .semaphore_empty
            .try_take_from_isr(higher_prio_task_waken.as_deref_mut())
        {
            if queue
                .mutex
                .try_take_from_isr(higher_prio_task_waken.as_deref_mut())
            {
                let inner = unsafe { &mut *queue.inner.get() };
                inner.send_to_back(item);

                queue
                    .mutex
                    .try_give_from_isr(higher_prio_task_waken.as_deref_mut());
                queue
                    .semaphore_full
                    .try_give_from_isr(higher_prio_task_waken);
                true
            } else {
                queue
                    .semaphore_empty
                    .try_give_from_isr(higher_prio_task_waken);
                false
            }
        } else {
            false
        }
    }

    // 698
    unsafe fn receive(queue: QueuePtr, item: *mut u8, timeout_us: Option<u32>) -> bool {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        if queue.semaphore_full.take(timeout_us) {
            if queue.mutex.take(timeout_us) {
                let inner = unsafe { &mut *queue.inner.get() };
                inner.read_from_front(item);

                queue.mutex.give();
                queue.semaphore_empty.give();
                true
            } else {
                queue.semaphore_full.give();
                false
            }
        } else {
            false
        }
    }

    // 718
    unsafe fn try_receive_from_isr(
        queue: QueuePtr,
        item: *mut u8,
        mut higher_prio_task_waken: Option<&mut bool>,
    ) -> bool {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        if queue
            .semaphore_full
            .try_take_from_isr(higher_prio_task_waken.as_deref_mut())
        {
            if queue
                .mutex
                .try_take_from_isr(higher_prio_task_waken.as_deref_mut())
            {
                let inner = unsafe { &mut *queue.inner.get() };
                inner.read_from_front(item);

                queue
                    .mutex
                    .try_give_from_isr(higher_prio_task_waken.as_deref_mut());
                queue
                    .semaphore_empty
                    .try_give_from_isr(higher_prio_task_waken);
                true
            } else {
                queue
                    .semaphore_full
                    .try_give_from_isr(higher_prio_task_waken);
                false
            }
        } else {
            false
        }
    }

    // 754
    unsafe fn remove(queue: QueuePtr, item: *const u8) {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        if queue.semaphore_full.take(Some(0)) {
            queue.mutex.take(None);

            let inner = unsafe { &mut *queue.inner.get() };
            let item_removed = inner.remove(item);

            queue.mutex.give();

            if item_removed {
                queue.semaphore_empty.give();
            } else {
                queue.semaphore_full.give();
            }
        }
    }

    // 773
    fn messages_waiting(queue: QueuePtr) -> usize {
        let queue = unsafe { CompatQueue::from_ptr(queue) };

        queue.semaphore_full.current_count() as usize
    }
}
