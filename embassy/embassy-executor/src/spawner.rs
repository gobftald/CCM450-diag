// 2
use core::marker::PhantomData;
use core::mem;

// 7
use super::raw;

/// Token to spawn a newly-created task in an executor.
///
/// When calling a task function (like `#[embassy_executor::task] async fn my_task() { ... }`), the returned
/// value is a `SpawnToken` that represents an instance of the task, ready to spawn. You must
/// then spawn it into an executor, typically with [`Spawner::spawn()`].
///
/// We did not implement generic parameter `S` which determines whether the task can be spawned in executors
/// in other threads or not, because we have no threads in our implementation.
///
/// # Panics
///
/// Dropping a SpawnToken instance panics. You may not "abort" spawning a task in this way.
/// Once you've invoked a task function and obtained a SpawnToken, you *must* spawn it.
#[must_use = "Calling a task function does nothing on its own. You must spawn the returned SpawnToken, typically with Spawner::spawn()"]
// 24
//pub struct SpawnToken<S> {
pub struct SpawnToken {
    raw_task: Option<raw::TaskRef>,
    //phantom: PhantomData<*mut S>,
}

// 29
impl SpawnToken {
    pub(crate) unsafe fn new(raw_task: raw::TaskRef) -> Self {
        Self {
            raw_task: Some(raw_task),
            //phantom: PhantomData,
        }
    }

    /// Return a SpawnToken that represents a failed spawn.
    // 47
    pub fn new_failed() -> Self {
        Self {
            raw_task: None,
            //phantom: PhantomData,
        }
    }
}

impl Drop for SpawnToken {
    fn drop(&mut self) {
        // TODO deallocate the task instead.
        panic!("SpawnToken instances may not be dropped. You must pass them to Spawner::spawn()")
    }
}

/// Error returned when spawning a task.
#[derive(Copy, Clone)]
// 64
pub enum SpawnError {
    /// Too many instances of this task are already running.
    ///
    /// By default, a task marked with `#[embassy_executor::task]` can only have one instance
    /// running at a time. You may allow multiple instances to run in parallel with
    /// `#[embassy_executor::task(pool_size = 4)]`, at the cost of higher RAM usage.
    Busy,
}

// 73
impl core::fmt::Debug for SpawnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

// 79
impl core::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SpawnError::Busy => write!(f, "Busy - Too many instances of this task are already running. Check the `pool_size` attribute of the task."),
        }
    }
}

// 87
#[cfg(feature = "defmt")]
impl defmt::Format for SpawnError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            SpawnError::Busy => defmt::write!(f, "Busy - Too many instances of this task are already running. Check the `pool_size` attribute of the task."),
        }
    }
}

/// Handle to spawn tasks into an executor.
#[derive(Copy, Clone)]
// 105
pub struct Spawner {
    executor: &'static raw::Executor,
    not_send: PhantomData<*mut ()>,
}

// 110
impl Spawner {
    pub(crate) fn new(executor: &'static raw::Executor) -> Self {
        Self {
            executor,
            not_send: PhantomData,
        }
    }

    /// Spawn a task into an executor.
    ///
    /// You obtain the `token` by calling a task function (i.e. one marked with `#[embassy_executor::task]`).
    // 144
    //pub fn spawn<S>(&self, token: SpawnToken<S>) -> Result<(), SpawnError> {
    pub fn spawn(&self, token: SpawnToken) -> Result<(), SpawnError> {
        let task = token.raw_task;
        mem::forget(token);

        match task {
            Some(task) => {
                unsafe { self.executor.spawn(task) };
                Ok(())
            }
            None => Err(SpawnError::Busy),
        }
    }

    // Used by the `embassy_executor_macros::main!` macro to throw an error when spawn
    // fails. This is here to allow conditional use of `defmt::unwrap!`
    // without introducing a `defmt` feature in the `embassy_executor_macros` package,
    // which would require use of `-Z namespaced-features`.
    /// Spawn a task into an executor, panicking on failure.
    ///
    /// # Panics
    ///
    /// Panics if the spawning fails.
    // 166
    pub fn must_spawn(&self, token: SpawnToken) {
        unwrap!(self.spawn(token));
    }
}
