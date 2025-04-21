use core::cell::Cell;

// 12
mod run_queue_critical_section;

// 17
mod state_critical_section;

// 19
pub mod timer_queue;
// 22
pub(crate) mod util;

// 30
use core::ptr::NonNull;

// 39
use self::run_queue_critical_section::{RunQueue, RunQueueItem};
use self::state_critical_section::State;
use self::util::UninitCell;

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

// 394
pub(crate) struct Executor {
    run_queue: RunQueue,
    pender: Pender,
}

// 399
impl Executor {
    // 400
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
    #[inline(always)]
    // 414
    unsafe fn enqueue(&self, task: TaskRef) {
        #[cfg(feature = "trace")]
        trace::task_ready_begin(self, &task);

        unsafe {
            if self.run_queue.enqueue(task) {
                self.pender.pend();
            }
        }
    }

    // 423
    pub(super) unsafe fn spawn(&'static self, task: TaskRef) {
        task.header()
            .executor
            //.store((self as *const Self).cast_mut(), Ordering::Relaxed);
            .set((self as *const Self).cast_mut());

        #[cfg(feature = "trace")]
        trace::task_new(self, &task);

        //state_critical_section::locked(|l| {
        //    self.enqueue(task, l);
        //})
        unsafe {
            self.enqueue(task);
        }
    }
}
