#[cfg_attr(not(target_has_atomic = "ptr"), path = "run_queue_critical_section.rs")]
mod run_queue;

#[cfg_attr(not(target_has_atomic = "8"), path = "state_critical_section.rs")]
mod state;

pub(crate) mod util;

use core::future::Future;
use core::marker::PhantomData;
use core::ptr::NonNull;

use self::run_queue::{RunQueue, RunQueueItem};
use self::state::State;
use self::util::{SyncUnsafeCell, UninitCell};
use super::SpawnToken;

/// Raw task header for use in task pointers.
pub(crate) struct TaskHeader {
    pub(crate) state: State,
    pub(crate) run_queue_item: RunQueueItem,
    pub(crate) executor: SyncUnsafeCell<Option<&'static SyncExecutor>>,
    poll_fn: SyncUnsafeCell<Option<unsafe fn(TaskRef)>>,

    #[cfg(feature = "integrated-timers")]
    pub(crate) expires_at: SyncUnsafeCell<u64>,
    #[cfg(feature = "integrated-timers")]
    pub(crate) timer_queue_item: timer_queue::TimerQueueItem,
}

/// This is essentially a `&'static TaskStorage<F>` where the type of the future has been erased.
#[derive(Clone, Copy)]
// 58
pub struct TaskRef {
    ptr: NonNull<TaskHeader>,
}

// 62
unsafe impl Send for TaskRef where &'static TaskHeader: Send {}
// we don't need it yet
//unsafe impl Sync for TaskRef where &'static TaskHeader: Sync {}

// 65
impl TaskRef {
    fn new<F: Future + 'static>(task: &'static TaskStorage<F>) -> Self {
        Self {
            ptr: NonNull::from(task).cast(),
        }
    }
}

/// Raw storage in which a task can be spawned.
///
/// This struct holds the necessary memory to spawn one task whose future is `F`.
/// At a given time, the `TaskStorage` may be in spawned or not-spawned state. You
/// may spawn it with [`TaskStorage::spawn()`], which will fail if it is already spawned.
///
/// A `TaskStorage` must live forever, it may not be deallocated even after the task has finished
/// running. Hence the relevant methods require `&'static self`. It may be reused, however.

// repr(C) is needed to guarantee that the Task is located at offset 0
// This makes it safe to cast between TaskHeader and TaskStorage pointers.
#[repr(C)]
// 105
pub struct TaskStorage<F: Future + 'static> {
    raw: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

// 110
impl<F: Future + 'static> TaskStorage<F> {
    // 111
    const NEW: Self = Self::new();

    /// Create a new TaskStorage, in not-spawned state.
    // 114
    pub const fn new() -> Self {
        Self {
            raw: TaskHeader {
                state: State::new(),
                run_queue_item: RunQueueItem::new(),
                executor: SyncUnsafeCell::new(None),
                // Note: this is lazily initialized so that a static `TaskStorage` will go in `.bss`
                poll_fn: SyncUnsafeCell::new(None),

                #[cfg(feature = "integrated-timers")]
                expires_at: SyncUnsafeCell::new(0),
                #[cfg(feature = "integrated-timers")]
                timer_queue_item: timer_queue::TimerQueueItem::new(),
            },
            future: UninitCell::uninit(),
        }
    }

    // 153
    unsafe fn poll(p: TaskRef) {
        /* ... */
    }
}

/// An uninitialized [`TaskStorage`].
// 185
pub struct AvailableTask<F: Future + 'static> {
    task: &'static TaskStorage<F>,
}

// 189
impl<F: Future + 'static> AvailableTask<F> {
    /// Try to claim a [`TaskStorage`].
    ///
    /// This function returns `None` if a task has already been spawned and has not finished running.
    // 193
    pub fn claim(task: &'static TaskStorage<F>) -> Option<Self> {
        task.raw.state.spawn().then(|| Self { task })
    }

    // 197
    fn initialize_impl<S>(self, future: impl FnOnce() -> F) -> SpawnToken<S> {
        unsafe {
            self.task.raw.poll_fn.set(Some(TaskStorage::<F>::poll));
            self.task.future.write_in_place(future);

            let task = TaskRef::new(self.task);

            SpawnToken::new(task)
        }
    }
}

/// Raw storage that can hold up to N tasks of the same type.
///
/// This is essentially a `[TaskStorage<F>; N]`.
// 253
pub struct TaskPool<F: Future + 'static, const N: usize> {
    pool: [TaskStorage<F>; N],
}

// 257
impl<F: Future + 'static, const N: usize> TaskPool<F, N> {
    /// Create a new TaskPool, with all tasks in non-spawned state.
    // 259
    pub const fn new() -> Self {
        Self {
            pool: [TaskStorage::NEW; N],
        }
    }

    // 265
    fn spawn_impl<T>(&'static self, future: impl FnOnce() -> F) -> SpawnToken<T> {
        match self.pool.iter().find_map(AvailableTask::claim) {
            Some(task) => task.initialize_impl::<T>(future),
            None => SpawnToken::new_failed(),
        }
    }

    /// Like spawn(), but allows the task to be send-spawned if the args are Send even if
    /// the future is !Send.
    ///
    /// SAFETY: `future` must be a closure of the form `move || my_async_fn(args)`, where `my_async_fn`
    /// is an `async fn`, NOT a hand-written `Future`.
    // 292
    pub unsafe fn _spawn_async_fn<FutFn>(&'static self, future: FutFn) -> SpawnToken<impl Sized>
    where
        FutFn: FnOnce() -> F,
    {
        // See the comment in AvailableTask::__initialize_async_fn for explanation.
        self.spawn_impl::<FutFn>(future)
    }
}

#[derive(Clone, Copy)]
// 301
pub(crate) struct Pender(*mut ());

// 34
// don't need it yet
//unsafe impl Send for Pender {}
unsafe impl Sync for Pender {}

// 316
pub(crate) struct SyncExecutor {
    run_queue: RunQueue,
    pender: Pender,

    #[cfg(feature = "integrated-timers")]
    pub(crate) timer_queue: timer_queue::TimerQueue,
    #[cfg(feature = "integrated-timers")]
    alarm: AlarmHandle,
}

// 326
impl SyncExecutor {
    // 327
    pub(crate) fn new(pender: Pender) -> Self {
        #[cfg(feature = "integrated-timers")]
        let alarm = unsafe { unwrap!(embassy_time_driver::allocate_alarm()) };

        Self {
            run_queue: RunQueue::new(),
            pender,

            #[cfg(feature = "integrated-timers")]
            timer_queue: timer_queue::TimerQueue::new(),
            #[cfg(feature = "integrated-timers")]
            alarm,
        }
    }

    // 364
    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        /*
        task.header().executor.set(Some(self));

        #[cfg(feature = "rtos-trace")]
        trace::task_new(task.as_ptr() as u32);

        self.enqueue(task);
        */
    }

    // 376
    pub(crate) unsafe fn poll(&'static self) {}
}

#[repr(transparent)]
// 472
pub struct Executor {
    pub(crate) inner: SyncExecutor,

    _not_sync: PhantomData<*mut ()>,
}

// 478
impl Executor {
    /// Create a new executor.
    ///
    /// When the executor has work to do, it will call the pender function and pass `context` to it.
    ///
    /// See [`Executor`] docs for details on the pender.
    // 488
    pub fn new(context: *mut ()) -> Self {
        Self {
            inner: SyncExecutor::new(Pender(context)),
            _not_sync: PhantomData,
        }
    }

    /// Spawn a task in this executor.
    ///
    /// # Safety
    ///
    /// `task` must be a valid pointer to an initialized but not-already-spawned task.
    ///
    /// It is OK to use `unsafe` to call this from a thread that's not the executor thread.
    /// In this case, the task's Future must be Send. This is because this is effectively
    /// sending the task to the executor thread.
    // 504
    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        self.inner.spawn(task)
    }

    /// Poll all queued tasks in this executor.
    ///
    /// This loops over all tasks that are queued to be polled (i.e. they're
    /// freshly spawned or they've been woken). Other tasks are not polled.
    ///
    /// You must call `poll` after receiving a call to the pender. It is OK
    /// to call `poll` even when not requested by the pender, but it wastes
    /// energy.
    ///
    pub unsafe fn poll(&'static self) {
        self.inner.poll()
    }

    /// Get a spawner that spawns tasks in this executor.
    ///
    /// It is OK to call this method multiple times to obtain multiple
    /// `Spawner`s. You may also copy `Spawner`s.
    // 533
    pub fn spawner(&'static self) -> super::Spawner {
        super::Spawner::new(self)
    }
}
