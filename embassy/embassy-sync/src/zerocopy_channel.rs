//! A zero-copy queue for sending values between asynchronous tasks.
//!
//! It can be used concurrently by a producer (sender) and a
//! consumer (receiver), i.e. it is an  "SPSC channel".
//!
//! This queue takes a Mutex type so that various
//! targets can be attained. For example, a ThreadModeMutex can be used
//! for single-core Cortex-M targets where messages are only passed
//! between tasks running in thread mode. Similarly, a CriticalSectionMutex
//! can also be used for single-core targets where messages are to be
//! passed from exception mode e.g. out of an interrupt handler.
//!
//! This module provides a bounded channel that has a limit on the number of
//! messages that it can store, and if this limit is reached, trying to send
//! another message will result in an error being returned.
//
// Based on our RawMutex, implemented in esp_hal::sync::RawMutex, it behaves
// like CriticalSectionMutex (single-core targets where messages can be
// passed out of an interrupt handler)

// 17
use core::cell::RefCell;
use core::future::{poll_fn, Future};
use core::marker::PhantomData;
use core::task::Poll;

// 22
use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::waitqueue::WakerRegistration;

/// A bounded zero-copy channel for communicating between asynchronous tasks
/// with backpressure.
///
/// The channel will buffer up to the provided number of messages.  Once the
/// buffer is full, attempts to `send` new messages will wait until a message is
/// received from the channel.
///
/// All data sent will become available in the same order as it was sent.
///
/// The channel requires a buffer of recyclable elements.  Writing to the channel is done through
/// an `&mut T`.
#[derive(Debug)]
// 38
pub struct Channel<'a, M: RawMutex, T> {
    buf: BufferPtr<T>,
    phantom: PhantomData<&'a mut T>,
    state: Mutex<M, RefCell<State>>,
}

// 44
impl<'a, M: RawMutex, T> Channel<'a, M, T> {
    /// Initialize a new [`Channel`].
    ///
    /// The provided buffer will be used and reused by the channel's logic, and thus dictates the
    /// channel's capacity.
    // 49
    pub fn new(buf: &'a mut [T]) -> Self {
        let len = buf.len();
        assert!(len != 0);

        Self {
            buf: BufferPtr(buf.as_mut_ptr()),
            phantom: PhantomData,
            state: Mutex::new(RefCell::new(State {
                capacity: len,
                front: 0,
                back: 0,
                full: false,
                send_waker: WakerRegistration::new(),
                receive_waker: WakerRegistration::new(),
            })),
        }
    }

    /// Creates a [`Sender`] and [`Receiver`] from an existing channel.
    ///
    /// Further Senders and Receivers can be created through [`Sender::borrow`] and
    /// [`Receiver::borrow`] respectively.
    // 71
    pub fn split(&mut self) -> (Sender<'_, M, T>, Receiver<'_, M, T>) {
        (Sender { channel: self }, Receiver { channel: self })
    }
}

#[repr(transparent)]
#[derive(Debug)]
// 100
struct BufferPtr<T>(*mut T);

// 102
impl<T> BufferPtr<T> {
    unsafe fn add(&self, count: usize) -> *mut T {
        self.0.add(count)
    }
}

/// Send-only access to a [`Channel`].
#[derive(Debug)]
// 113
pub struct Sender<'a, M: RawMutex, T> {
    channel: &'a Channel<'a, M, T>,
}

// 117
impl<'a, M: RawMutex, T> Sender<'a, M, T> {
    /// Asynchronously send a value over the channel.
    // 149
    pub fn send(&mut self) -> impl Future<Output = &mut T> {
        poll_fn(|cx| {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.push_index() {
                    Some(i) => {
                        let r = unsafe { &mut *self.channel.buf.add(i) };
                        Poll::Ready(r)
                    }
                    None => {
                        s.receive_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
    }

    /// Notify the channel that the sending of the value has been finalized.
    // 168
    pub fn send_done(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().push_done())
    }
}

/// Receive-only access to a [`Channel`].
#[derive(Debug)]
// 197
pub struct Receiver<'a, M: RawMutex, T> {
    channel: &'a Channel<'a, M, T>,
}

// 201
impl<'a, M: RawMutex, T> Receiver<'a, M, T> {
    // 233
    /// Asynchronously receive a value over the channel.
    pub fn receive(&mut self) -> impl Future<Output = &mut T> {
        poll_fn(|cx| {
            self.channel.state.lock(|s| {
                let s = &mut *s.borrow_mut();
                match s.pop_index() {
                    Some(i) => {
                        let r = unsafe { &mut *self.channel.buf.add(i) };
                        Poll::Ready(r)
                    }
                    None => {
                        s.send_waker.register(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
    }

    /// Notify the channel that the receiving of the value has been finalized.
    // 252
    pub fn receive_done(&mut self) {
        self.channel.state.lock(|s| s.borrow_mut().pop_done())
    }
}

unsafe impl<T> Send for BufferPtr<T> {}
unsafe impl<T> Sync for BufferPtr<T> {}

#[derive(Debug)]
// 280
struct State {
    /// Maximum number of elements the channel can hold.
    capacity: usize,

    /// Front index. Always 0..=(N-1)
    front: usize,
    /// Back index. Always 0..=(N-1).
    back: usize,

    /// Used to distinguish "empty" and "full" cases when `front == back`.
    /// May only be `true` if `front == back`, always `false` otherwise.
    full: bool,

    send_waker: WakerRegistration,
    receive_waker: WakerRegistration,
}

// 297
impl State {
    // 298
    fn increment(&self, i: usize) -> usize {
        if i + 1 == self.capacity {
            0
        } else {
            i + 1
        }
    }

    // 327
    fn is_full(&self) -> bool {
        self.full
    }

    // 331
    fn is_empty(&self) -> bool {
        self.front == self.back && !self.full
    }

    // 335
    fn push_index(&mut self) -> Option<usize> {
        match self.is_full() {
            true => None,
            false => Some(self.back),
        }
    }

    // 342
    fn push_done(&mut self) {
        assert!(!self.is_full());
        self.back = self.increment(self.back);
        if self.back == self.front {
            self.full = true;
        }
        self.send_waker.wake();
    }

    // 351
    fn pop_index(&mut self) -> Option<usize> {
        match self.is_empty() {
            true => None,
            false => Some(self.front),
        }
    }

    // 258
    fn pop_done(&mut self) {
        assert!(!self.is_empty());
        self.front = self.increment(self.front);
        self.full = false;
        self.receive_waker.wake();
    }
}
