use core::task::Waker;

/// Utility struct to register and wake a waker.
#[derive(Debug)]
// 5
pub struct WakerRegistration {
    waker: Option<Waker>,
}

// 9
impl WakerRegistration {
    // 10
    pub const fn new() -> Self {
        Self { waker: None }
    }

    /// Wake the registered waker, if any.
    // 30
    pub fn wake(&mut self) {
        self.waker.take().map(|w| w.wake());
    }
}
