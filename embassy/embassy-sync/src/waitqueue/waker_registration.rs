// 2
use core::task::Waker;

/// Utility struct to register and wake a waker.
/// If a waker is registered, registering another waker will replace the previous one.
/// The previous waker will be woken in this case, giving it a chance to reregister itself.
/// Although it is possible to wake multiple tasks this way,
/// this will cause them to wake each other in a loop registering themselves.
#[derive(Debug, Default)]
// 10
pub struct WakerRegistration {
    waker: Option<Waker>,
}

// 14
impl WakerRegistration {
    /// Create a new `WakerRegistration`.
    // 16
    pub const fn new() -> Self {
        Self { waker: None }
    }

    /// Wake the registered waker, if any.
    // 46
    pub fn wake(&mut self) {
        if let Some(w) = self.waker.take() {
            w.wake()
        }
    }
}
