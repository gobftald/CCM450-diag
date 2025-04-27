//! Raw executor.
//!
//! This module exposes "raw" Executor and Task structs for more low level control.
//!
//! ## WARNING: here be dragons!
//!
//! Using this module requires respecting subtle safety contracts. If you can, prefer using the safe
//! [executor wrappers](crate::Executor) and the [`embassy_executor::task`](embassy_executor_macros::task) macro, which are fully safe.

use core::cell::Cell;

// 12
mod run_queue_critical_section;

// 17
mod state_critical_section;

// 19
pub mod timer_queue;
// 22
pub(crate) mod util;
//#[cfg_attr(feature = "turbowakers", path = "waker_turbo.rs")]
mod waker;

// 26
use core::future::Future;
// 28
use core::mem;
use core::pin::Pin;

// 30
use core::ptr::NonNull;
// 34
use core::task::{Context, Poll};

// 39
use self::run_queue_critical_section::{RunQueue, RunQueueItem};
use self::state_critical_section::State;
use self::util::UninitCell;
// 43
use super::SpawnToken;

// 84
pub(crate) struct TaskHeader {
    pub(crate) state: State,
    pub(crate) run_queue_item: RunQueueItem,
    //pub(crate) executor: AtomicPtr<SyncExecutor>,
    // we drop syncness and atomic behaviour
    pub(crate) executor: Cell<*mut Executor>,
    //poll_fn: SyncUnsafeCell<Option<unsafe fn(TaskRef)>
    // we drop syncness
    // 'cell' gives potentially more function then SyncUnsafeCell, but its code size should be the same
    poll_fn: Cell<Option<unsafe fn(TaskRef)>>,

    /// Integrated timer queue storage. This field should not be accessed outside of the timer queue.
    pub(crate) timer_queue_item: timer_queue::TimerQueueItem,
}

/// This is essentially a `&'static TaskStorage<F>` where the type of the future has been erased.
#[derive(Clone, Copy, PartialEq)]
// 96
pub struct TaskRef {
    ptr: NonNull<TaskHeader>,
}
// NonNull is a *mut T but non-zero and covariant (it is not interior mutable)
// makes castings easier then using *mut
// e.g. in its new() it simple calls cast() to "erease type of future" and points only TaskHeader
// e.g. 'wakers' are always created with `TaskRef::as_ptr` which is *const TaskHeader
// then casting between *const pointer is easy so
// in 'fn wake(p: *const ())' we use wake_task(TaskRef::from_ptr(p as *const TaskHeader))
// or in 'fn task_from_waker(waker: &Waker) -> TaskRef' TaskRef::from_ptr(waker.data() as *const TaskHeader)
// with its dangling function it also provide some type of deinit feature
// in sumary it is *mut (for cross/walking diffrent type) building an easier and safer casting functionalities

// 100
//unsafe impl Send for TaskRef where &'static TaskHeader: Send {}
//unsafe impl Sync for TaskRef where &'static TaskHeader: Sync {}

// 103
impl TaskRef {
    // 104
    fn new<F: Future + 'static>(task: &'static TaskStorage<F>) -> Self {
        Self {
            ptr: NonNull::from(task).cast(),
        }
    }

    /// Safety: The pointer must have been obtained with `Task::as_ptr`
    // 111
    pub(crate) unsafe fn from_ptr(ptr: *const TaskHeader) -> Self {
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(ptr as *mut TaskHeader),
            }
        }
    }

    /// # Safety
    ///
    /// The result of this function must only be compared
    /// for equality, or stored, but not used.
    // 121
    pub const unsafe fn dangling() -> Self {
        Self {
            ptr: NonNull::dangling(),
        }
    }

    // 127
    pub(crate) fn header(self) -> &'static TaskHeader {
        unsafe { self.ptr.as_ref() }
    }

    /// The returned pointer is valid for the entire TaskStorage.
    // 142
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
#[repr(C)]
// 170
pub struct TaskStorage<F: Future + 'static> {
    raw: TaskHeader,
    future: UninitCell<F>, // Valid if STATE_SPAWNED
}

// 175
unsafe fn poll_exited(_p: TaskRef) {
    // Nothing to do, the task is already !SPAWNED and dequeued.
}

// struct UninitCell<T>(MaybeUninit<UnsafeCell<T>>)
// why this contruction used here:
// MaybeUninit ensures the option like behaviour without overhead (simple low level ptr logic, no unwrap,
// or let Some(...), so it has an uninit (Some) state ensuring that it is UB if it used without init
// (containing a valid/spawned future)
// so it should be inited or rewriten (rewriten if 'static TaskStorage is reused)
// UnsafeCell provides the interior mutability
// in summary it is a lightweigth interior mutable rewritable Option

// 179
impl<F: Future + 'static> TaskStorage<F> {
    const NEW: Self = Self::new();

    /// Create a new TaskStorage, in not-spawned state.
    pub const fn new() -> Self {
        Self {
            raw: TaskHeader {
                state: State::new(),
                run_queue_item: RunQueueItem::new(),
                //executor: AtomicPtr::new(core::ptr::null_mut()),
                executor: Cell::new(core::ptr::null_mut()),
                // Note: this is lazily initialized so that a static `TaskStorage` will go in `.bss`
                poll_fn: Cell::new(None),

                timer_queue_item: timer_queue::TimerQueueItem::new(),
            },
            future: UninitCell::uninit(),
        }
    }

    // 219
    unsafe fn poll(p: TaskRef) {
        let this = &*p.as_ptr().cast::<TaskStorage<F>>();

        let future = Pin::new_unchecked(this.future.as_mut());
        let waker = waker::from_task(p);
        let mut cx = Context::from_waker(&waker);
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
// 263
pub struct AvailableTask<F: Future + 'static> {
    task: &'static TaskStorage<F>,
}

// 267
impl<F: Future + 'static> AvailableTask<F> {
    /// Try to claim a [`TaskStorage`].
    ///
    /// This function returns `None` if a task has already been spawned and has not finished running.
    // 271
    pub fn claim(task: &'static TaskStorage<F>) -> Option<Self> {
        // If task is idle, mark it as spawned + run_queued and return true
        task.raw.state.spawn().then(|| Self { task })
    }

    // 275
    fn initialize_impl<S>(self, future: impl FnOnce() -> F) -> SpawnToken {
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
// 331
pub struct TaskPool<F: Future + 'static, const N: usize> {
    pool: [TaskStorage<F>; N],
}

// 335
impl<F: Future + 'static, const N: usize> TaskPool<F, N> {
    /// Create a new TaskPool, with all tasks in non-spawned state.
    pub const fn new() -> Self {
        Self {
            pool: [TaskStorage::NEW; N],
        }
    }

    // 343
    //fn spawn_impl<T>(&'static self, future: impl FnOnce() -> F) -> SpawnToken<T> {
    fn spawn_impl<T>(&'static self, future: impl FnOnce() -> F) -> SpawnToken {
        match self.pool.iter().find_map(AvailableTask::claim) {
            Some(task) => task.initialize_impl::<T>(future),
            None => SpawnToken::new_failed(),
        }
    }

    /// Try to spawn a task in the pool.
    ///
    /// See [`TaskStorage::spawn()`] for details.
    ///
    /// This will loop over the pool and spawn the task in the first storage that
    /// is currently free. If none is free, a "poisoned" SpawnToken is returned,
    /// which will cause [`Spawner::spawn()`](super::Spawner::spawn) to return the error.
    // 357
    //pub fn spawn(&'static self, future: impl FnOnce() -> F) -> SpawnToken<impl Sized> {
    pub fn spawn(&'static self, future: impl FnOnce() -> F) -> SpawnToken {
        self.spawn_impl::<F>(future)
    }

    /// Like spawn(), but allows the task to be send-spawned if the args are Send even if
    /// the future is !Send.
    ///
    /// Not covered by semver guarantees. DO NOT call this directly. Intended to be used
    /// by the Embassy macros ONLY.
    ///
    /// SAFETY: `future` must be a closure of the form `move || my_async_fn(args)`, where `my_async_fn`
    /// is an `async fn`, NOT a hand-written `Future`.
    #[doc(hidden)]
    // 370
    //pub unsafe fn _spawn_async_fn<FutFn>(&'static self, future: FutFn) -> SpawnToken<impl Sized>
    pub unsafe fn _spawn_async_fn<FutFn>(&'static self, future: FutFn) -> SpawnToken
    where
        FutFn: FnOnce() -> F,
    {
        // See the comment in AvailableTask::__initialize_async_fn for explanation.
        self.spawn_impl::<FutFn>(future)
    }
}

#[derive(Clone, Copy)]
// 380
pub(crate) struct Pender(*mut ());

// 385
impl Pender {
    pub(crate) fn pend(self) {
        unsafe extern "Rust" {
            fn __pender(context: *mut ());
        }
        unsafe { __pender(self.0) };
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
/// (in risc32 it is manged by 'static mut SIGNAL_WORK_THREAD_MODE: bool')
///
/// The pender function must be exported with the name `__pender` and have the following signature:
///
/// ```rust
/// #[export_name = "__pender"]
/// fn pender(context: *mut ()) {
///    // schedule `poll()` to be called
///
///
/// }
/// ```
///
/// The `context` argument is a piece of arbitrary data the executor will pass to the pender.
/// You can set the `context` when calling [`Executor::new()`]. You can use it to, (((for example,
/// differentiate between executors))), or to pass a pointer to a callback that should be called.
// 394
pub struct Executor {
    run_queue: RunQueue, // since run_queue is unsync, thus Executor is unsync as well
    pender: Pender,
}

// 399
impl Executor {
    // 400
    pub fn new(context: *mut ()) -> Self {
        Self {
            run_queue: RunQueue::new(),
            pender: Pender(context),
        }
    }

    /// Enqueue a task in the task queue
    ///
    /// # Safety
    /// - `task` must be a valid pointer to a spawned task.
    /// - `task` must be set up to run in this executor.
    /// - `task` must NOT be already enqueued (in this executor or another one).
    #[inline(always)]
    // 414
    unsafe fn enqueue(&self, task: TaskRef) {
        #[cfg(feature = "trace")]
        trace::task_ready_begin(self, &task);

        unsafe {
            // (only) insert task into RunQueue
            if self.run_queue.enqueue(task) {
                // schedule `poll()` to be called
                self.pender.pend();
            }
        }
    }

    /// Spawn a task in this executor.
    ///
    /// # Safety
    ///
    /// `task` must be a valid pointer to an initialized but not-already-spawned task.
    // 423
    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        task.header()
            .executor
            //.store((self as *const Self).cast_mut(), Ordering::Relaxed);
            // put itself to Task's 'executor' field
            .set((self as *const Self).cast_mut());

        #[cfg(feature = "trace")]
        trace::task_new(self, &task);

        //state_critical_section::locked(|l| {
        //    self.enqueue(task, l);
        //})
        unsafe {
            // insert task into RunQueue then call 'pend' (schedule `poll()` to be called)
            self.enqueue(task);
        }
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
    // in riscv32 it is managed by 'static mut SIGNAL_WORK_THREAD_MODE: bool'
    // 439
    pub unsafe fn poll(&'static self) {
        #[cfg(feature = "trace")]
        trace::poll_start(self);

        self.run_queue.dequeue_all(|p| {
            let task = p.header();

            #[cfg(feature = "trace")]
            trace::task_exec_begin(self, &p);

            // Run the task
            unsafe {
                task.poll_fn.get().unwrap_unchecked()(p);
            }

            #[cfg(feature = "trace")]
            trace::task_exec_end(self, &p);
        });

        #[cfg(feature = "trace")]
        trace::executor_idle(self)
    }

    /// Get a spawner that spawns tasks in this executor.
    ///
    /// It is OK to call this method multiple times to obtain multiple
    /// `Spawner`s. You may also copy `Spawner`s.
    pub fn spawner(&'static self) -> super::Spawner {
        super::Spawner::new(self)
    }
}

/// Wake a task by `TaskRef`.
///
/// You can obtain a `TaskRef` from a `Waker` using [`task_from_waker`].
// 573
pub fn wake_task(task: TaskRef) {
    let header = task.header();
    //header.state.run_enqueue(|l| {
    header.state.run_enqueue(|| {
        // We have just marked the task as scheduled, so enqueue it.
        unsafe {
            let executor = header
                .executor
                //.load(Ordering::Relaxed)
                //.as_ref()
                //.unwrap_unchecked();
                .get();
            //executor).enqueue(task, l);
            (*executor).enqueue(task);
        }
    });
}

/// Wake a task by `TaskRef` without calling pend.
///
/// You can obtain a `TaskRef` from a `Waker` using [`task_from_waker`].
pub fn wake_task_no_pend(task: TaskRef) {
    let header = task.header();
    header.state.run_enqueue(|| {
        // We have just marked the task as scheduled, so enqueue it.
        unsafe {
            let executor = header
                .executor
                //.load(Ordering::Relaxed)
                //.as_ref()
                //.unwrap_unchecked();
                .get();
            //executor.run_queue.enqueue(task, l);
            (*executor).run_queue.enqueue(task);
        }
    });
}
