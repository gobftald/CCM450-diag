//! Raw executor.
//!
//! This module exposes "raw" Executor and Task structs for more low level control.
//!
//! ## WARNING: here be dragons!
//!
//! Using this module requires respecting subtle safety contracts. If you can, prefer using the safe
//! [executor wrappers](crate::Executor) and the [`embassy_executor::task`](embassy_executor_macros::task) macro, which are fully safe.

// 10
#[cfg_attr(target_has_atomic = "ptr", path = "run_queue_atomics.rs")]
#[cfg_attr(not(target_has_atomic = "ptr"), path = "run_queue_critical_section.rs")]
mod run_queue;

// 14
#[cfg_attr(all(cortex_m, target_has_atomic = "32"), path = "state_atomics_arm.rs")]
#[cfg_attr(all(not(cortex_m), target_has_atomic = "8"), path = "state_atomics.rs")]
#[cfg_attr(not(target_has_atomic = "8"), path = "state_critical_section.rs")]
mod state;

// 19
#[cfg(feature = "trace")]
pub mod trace;

// 22
pub(crate) mod util;
//#[cfg_attr(feature = "turbowakers", path = "waker_turbo.rs")]
mod waker;

// 25
use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::ptr::NonNull;
#[cfg(not(feature = "arch-avr"))]
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering;
use core::task::{Context, Poll, Waker};

// 35
use embassy_executor_timer_queue::TimerQueueItem;
#[cfg(feature = "arch-avr")]
use portable_atomic::AtomicPtr;

// 39
use self::run_queue::{RunQueue, RunQueueItem};
use self::state::State;
use self::util::{SyncUnsafeCell, UninitCell};
pub use self::waker::task_from_waker;
use super::SpawnToken;
use crate::SpawnError;

// 45
#[unsafe(no_mangle)]
extern "Rust" fn __embassy_time_queue_item_from_waker(
    waker: &Waker,
) -> &'static mut TimerQueueItem {
    task_from_waker(waker).timer_queue_item()
}

/// Raw task header for use in task pointers.
///
/// A task can be in one of the following states:
///
/// - Not spawned: the task is ready to spawn.
/// - `SPAWNED`: the task is currently spawned and may be running.
/// - `RUN_ENQUEUED`: the task is enqueued to be polled. Note that the task may be `!SPAWNED`.
///    In this case, the `RUN_ENQUEUED` state will be cleared when the task is next polled, without
///    polling the task's future.
///
/// A task's complete life cycle is as follows:
///
/// ```text
/// ┌────────────┐   ┌────────────────────────┐
/// │Not spawned │◄─5┤Not spawned|Run enqueued│
/// │            ├6─►│                        │
/// └─────┬──────┘   └──────▲─────────────────┘
///       1                 │
///       │    ┌────────────┘
///       │    4
/// ┌─────▼────┴─────────┐
/// │Spawned|Run enqueued│
/// │                    │
/// └─────┬▲─────────────┘
///       2│
///       │3
/// ┌─────▼┴─────┐
/// │  Spawned   │
/// │            │
/// └────────────┘
/// ```
///
/// Transitions:
/// - 1: Task is spawned - `AvailableTask::claim -> Executor::spawn`
/// - 2: During poll - `RunQueue::dequeue_all -> State::run_dequeue`
/// - 3: Task wakes itself, waker wakes task, or task exits - `Waker::wake -> wake_task -> State::run_enqueue`
/// - 4: A run-queued task exits - `TaskStorage::poll -> Poll::Ready`
/// - 5: Task is dequeued. The task's future is not polled, because exiting the task replaces its `poll_fn`.
/// - 6: A task is waken when it is not spawned - `wake_task -> State::run_enqueue`
// 89
pub(crate) struct TaskHeader {
    pub(crate) state: State,
    pub(crate) run_queue_item: RunQueueItem,

    pub(crate) executor: AtomicPtr<SyncExecutor>,
    poll_fn: SyncUnsafeCell<Option<unsafe fn(TaskRef)>>,

    /// Integrated timer queue storage. This field should not be accessed outside of the timer queue.
    pub(crate) timer_queue_item: TimerQueueItem,

    #[cfg(feature = "trace")]
    pub(crate) name: Option<&'static str>,
    #[cfg(feature = "trace")]
    pub(crate) id: u32,
    #[cfg(feature = "trace")]
    all_tasks_next: AtomicPtr<TaskHeader>,
}

/// This is essentially a `&'static TaskStorage<F>` where the type of the future has been erased.
// 106
#[derive(Clone, Copy, PartialEq)]
pub struct TaskRef {
    ptr: NonNull<TaskHeader>, // !Sync and !Send
}

// 111
unsafe impl Send for TaskRef where &'static TaskHeader: Send {}
unsafe impl Sync for TaskRef where &'static TaskHeader: Sync {}

// 114
impl TaskRef {
    fn new<F: Future + 'static>(task: &'static TaskStorage<F>) -> Self {
        Self {
            ptr: NonNull::from(task).cast(),
        }
    }

    /// Safety: The pointer must have been obtained with `Task::as_ptr`
    // 122
    pub(crate) unsafe fn from_ptr(ptr: *const TaskHeader) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut TaskHeader),
        }
    }

    // 128
    pub(crate) fn header(self) -> &'static TaskHeader {
        unsafe { self.ptr.as_ref() }
    }

    /// Returns a reference to the timer queue item.
    // 143
    pub fn timer_queue_item(mut self) -> &'static mut TimerQueueItem {
        unsafe { &mut self.ptr.as_mut().timer_queue_item }
    }

    /// The returned pointer is valid for the entire TaskStorage.
    // 148
    pub(crate) fn as_ptr(self) -> *const TaskHeader {
        self.ptr.as_ptr()
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
///
/// Internally, the [embassy_executor::task](embassy_executor_macros::task) macro allocates an array of `TaskStorage`s
/// in a `static`. The most common reason to use the raw `Task` is to have control of where
/// the memory for the task is allocated: on the stack, or on the heap with e.g. `Box::leak`, etc.
// repr(C) is needed to guarantee that the Task is located at offset 0
// This makes it safe to cast between TaskHeader and TaskStorage pointers.
// 168
#[repr(C)]
pub struct TaskStorage<F: Future + 'static> {
    raw: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

// 174
unsafe fn poll_exited(_p: TaskRef) {
    // Nothing to do, the task is already !SPAWNED and dequeued.
}

// 178
impl<F: Future + 'static> TaskStorage<F> {
    const NEW: Self = Self::new();

    /// Create a new TaskStorage, in not-spawned state.
    // 182
    pub const fn new() -> Self {
        Self {
            raw: TaskHeader {
                state: State::new(),
                run_queue_item: RunQueueItem::new(),
                executor: AtomicPtr::new(core::ptr::null_mut()),
                // Note: this is lazily initialized so that a static `TaskStorage` will go in `.bss`
                poll_fn: SyncUnsafeCell::new(None),

                timer_queue_item: TimerQueueItem::new(),

                #[cfg(feature = "trace")]
                name: None,
                #[cfg(feature = "trace")]
                id: 0,
                #[cfg(feature = "trace")]
                all_tasks_next: AtomicPtr::new(core::ptr::null_mut()),
            },
            future: UninitCell::uninit(),
        }
    }

    // top (=task) level poll -> polls the task's future
    // 224
    unsafe fn poll(p: TaskRef) {
        let this = &*p.as_ptr().cast::<TaskStorage<F>>();

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = waker::from_task(p);
        let mut cx = Context::from_waker(&waker);

        // polls the task's future
        match future.poll(&mut cx) {
            Poll::Ready(_) => {
                #[cfg(feature = "trace")]
                let exec_ptr: *const SyncExecutor = this.raw.executor.load(Ordering::Relaxed);

                // As the future has finished and this function will not be called
                // again, we can safely drop the future here.
                this.future.drop_in_place();

                // We replace the poll_fn with a despawn function, so that the task is cleaned up
                // when the executor polls it next.
                this.raw.poll_fn.set(Some(poll_exited));

                // Make sure we despawn last, so that other threads can only spawn the task
                // after we're done with it.
                this.raw.state.despawn(); // &= !STATE_SPAWNED

                #[cfg(feature = "trace")]
                trace::task_end(exec_ptr, &p);
            }
            Poll::Pending => {}
        }

        // the compiler is emitting a virtual call for waker drop, but we know
        // it's a noop for our waker.
        mem::forget(waker);
    }
}

/// An uninitialized [`TaskStorage`].
// 268
pub struct AvailableTask<F: Future + 'static> {
    task: &'static TaskStorage<F>,
}

// 272
impl<F: Future + 'static> AvailableTask<F> {
    /// Try to claim a [`TaskStorage`].
    ///
    /// This function returns `None` if a task has already been spawned and has not finished running.
    pub fn claim(task: &'static TaskStorage<F>) -> Option<Self> {
        // If task is idle, mark it as spawned + run_queued and return true
        task.raw.state.spawn().then(|| Self { task })
    }

    // 280
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
// 336
pub struct TaskPool<F: Future + 'static, const N: usize> {
    pool: [TaskStorage<F>; N],
}

// 318
impl<F: Future + 'static, const N: usize> TaskPool<F, N> {
    /// Create a new TaskPool, with all tasks in non-spawned state.
    // 320
    pub const fn new() -> Self {
        Self {
            pool: [TaskStorage::NEW; N],
        }
    }

    /// Try to spawn a task in the pool.
    ///
    /// See [`TaskStorage::spawn()`] for details.
    ///
    /// This will loop over the pool and spawn the task in the first storage that
    /// is currently free. If none is free, a "poisoned" SpawnToken is returned,
    /// which will cause [`Spawner::spawn()`](super::Spawner::spawn) to return the error.
    // 348
    fn spawn_impl<T>(&'static self, future: impl FnOnce() -> F) -> SpawnToken<T> {
        match self.pool.iter().find_map(AvailableTask::claim) {
            Some(task) => task.initialize_impl::<T>(future),
            None => SpawnToken::new_failed(),
        }
    }

    /// Like spawn(), but allows the task to be send-spawned if the args are Send even if
    /// the future is !Send.
    ///
    /// Not covered by semver guarantees. DO NOT call this directly. Intended to be used
    /// by the Embassy macros ONLY.
    ///
    /// SAFETY: `future` must be a closure of the form `move || my_async_fn(args)`, where `my_async_fn`
    /// is an `async fn`, NOT a hand-written `Future`.
    // 375
    pub unsafe fn _spawn_async_fn<FutFn>(&'static self, future: FutFn) -> SpawnToken<impl Sized>
    where
        FutFn: FnOnce() -> F,
    {
        // See the comment in AvailableTask::__initialize_async_fn for explanation.
        self.spawn_impl::<FutFn>(future)
    }
}

// 384
#[derive(Clone, Copy)]
pub(crate) struct Pender(*mut ());

// 387
unsafe impl Send for Pender {}
unsafe impl Sync for Pender {}

// 390
impl Pender {
    pub(crate) fn pend(self) {
        extern "Rust" {
            fn __pender(context: *mut ());
        }
        unsafe { __pender(self.0) };
    }
}

// 399
pub(crate) struct SyncExecutor {
    run_queue: RunQueue,
    pender: Pender,
}

// 404
impl SyncExecutor {
    pub(crate) fn new(pender: Pender) -> Self {
        Self {
            run_queue: RunQueue::new(),
            pender,
        }
    }

    /// Enqueue a task in the task queue
    ///
    /// # Safety
    /// - `task` must be a valid pointer to a spawned task.
    /// - `task` must be set up to run in this executor.
    /// - `task` must NOT be already enqueued (in this executor or another one).
    // 418
    #[inline(always)]
    unsafe fn enqueue(&self, task: TaskRef, l: state::Token) {
        #[cfg(feature = "trace")]
        trace::task_ready_begin(self, &task);

        // (only) insert task into RunQueue
        if self.run_queue.enqueue(task, l) {
            // schedule `poll()` to be called
            self.pender.pend();
        }
    }

    // 428
    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        // put itself to Task's 'executor' field
        task.header()
            .executor
            .store((self as *const Self).cast_mut(), Ordering::Relaxed);

        #[cfg(feature = "trace")]
        trace::task_new(self, &task);

        state::locked(|l| {
            self.enqueue(task, l);
        })
    }

    /// # Safety
    ///
    /// Same as [`Executor::poll`], plus you must only call this on the thread this executor was created.
    // 444
    pub(crate) unsafe fn poll(&'static self) {
        #[cfg(feature = "trace")]
        trace::poll_start(self);

        self.run_queue.dequeue_all(|p| {
            let task = p.header();

            #[cfg(feature = "trace")]
            trace::task_exec_begin(self, &p);

            // Run the task
            task.poll_fn.get().unwrap_unchecked()(p);

            #[cfg(feature = "trace")]
            trace::task_exec_end(self, &p);
        });

        #[cfg(feature = "trace")]
        trace::executor_idle(self)
    }
}

/// Raw executor.
///
/// This is the core of the Embassy executor. It is low-level, requiring manual
/// handling of wakeups and task polling. If you can, prefer using one of the
/// [higher level executors](crate::Executor).
///
/// The raw executor leaves it up to you to handle wakeups and scheduling:
///
/// - To get the executor to do work, call `poll()`. This will poll all queued tasks (all tasks
///   that "want to run").
/// - You must supply a pender function, as shown below. The executor will call it to notify you
///   it has work to do. You must arrange for `poll()` to be called as soon as possible.
/// - Enabling `arch-xx` features will define a pender function for you. This means that you
///   are limited to using the executors provided to you by the architecture/platform
///   implementation. If you need a different executor, you must not enable `arch-xx` features.
///
/// The pender can be called from *any* context: any interrupt priority level, etc.
/// It may be called synchronously from any `Executor` method call as well.
/// You must deal with this correctly.
///
/// In particular, you must NOT call `poll` directly from the pender callback, as this violates
/// the requirement for `poll` to not be called reentrantly.
///
/// The pender function must be exported with the name `__pender` and have the following signature:
///
/// ```rust
/// #[export_name = "__pender"]
/// fn pender(context: *mut ()) {
///    // schedule `poll()` to be called
/// }
/// ```
///
/// The `context` argument is a piece of arbitrary data the executor will pass to the pender.
/// You can set the `context` when calling [`Executor::new()`]. You can use it to, (((for example,
/// differentiate between executors))), or to pass a pointer to a callback that should be called.
// 501
#[repr(transparent)]
pub struct Executor {
    pub(crate) inner: SyncExecutor,

    _not_sync: PhantomData<*mut ()>,
}

// 508
impl Executor {
    pub(crate) unsafe fn wrap(inner: &SyncExecutor) -> &Self {
        mem::transmute(inner)
    }

    /// Create a new executor.
    ///
    /// When the executor has work to do, it will call the pender function and pass `context` to it.
    ///
    /// See [`Executor`] docs for details on the pender.
    // 518
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
    //534
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
    /// # Safety
    ///
    /// You must call `initialize` before calling this method.
    ///
    /// You must NOT call `poll` reentrantly on the same executor.
    ///
    /// In particular, note that `poll` may call the pender synchronously. Therefore, you
    /// must NOT directly call `poll()` from the pender callback. Instead, the callback has to
    /// somehow schedule for `poll()` to be called later, at a time you know for sure there's
    /// no `poll()` already running.
    // 557
    pub unsafe fn poll(&'static self) {
        self.inner.poll()
    }

    /// Get a spawner that spawns tasks in this executor.
    ///
    /// It is OK to call this method multiple times to obtain multiple
    /// `Spawner`s. You may also copy `Spawner`s.
    // 565
    pub fn spawner(&'static self) -> super::Spawner {
        super::Spawner::new(self)
    }
}

/// Wake a task by `TaskRef`.
///
/// You can obtain a `TaskRef` from a `Waker` using [`task_from_waker`].
// 578
pub fn wake_task(task: TaskRef) {
    let header = task.header();
    header.state.run_enqueue(|l| {
        // We have just marked the task as scheduled, so enqueue it.
        unsafe {
            let executor = header
                .executor
                .load(Ordering::Relaxed)
                .as_ref()
                .unwrap_unchecked();
            executor.enqueue(task, l);
        }
    });
}

/// Wake a task by `TaskRef` without calling pend.
///
/// You can obtain a `TaskRef` from a `Waker` using [`task_from_waker`].
// 592
pub fn wake_task_no_pend(task: TaskRef) {
    let header = task.header();
    header.state.run_enqueue(|l| {
        // We have just marked the task as scheduled, so enqueue it.
        unsafe {
            let executor = header
                .executor
                .load(Ordering::Relaxed)
                .as_ref()
                .unwrap_unchecked();
            executor.run_queue.enqueue(task, l);
        }
    });
}
