// 1
#[cfg_attr(riscv, path = "riscv.rs")]
#[cfg_attr(xtensa, path = "xtensa.rs")]
pub(crate) mod arch_specific;

// 7
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

// 11
pub(crate) use arch_specific::*;
use esp_hal::system::Cpu;

// 22
#[cfg(feature = "esp-radio")]
use crate::semaphore::Semaphore;
//23
use crate::{run_queue::Priority, wait_queue::WaitQueue};

// 32
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum TaskState {
    Ready,
    Sleeping,
    Deleted,
}

// 40
pub(crate) type TaskPtr = NonNull<Task>;
pub(crate) type TaskListItem = Option<TaskPtr>;

// 44
/// An abstraction that allows the task to contain multiple different queue pointers.
pub(crate) trait TaskListElement: Default {
    /// Returns the pointer to the next element in the list.
    fn next(task: TaskPtr) -> Option<TaskPtr>;

    /// Sets the pointer to the next element in the list.
    fn set_next(task: TaskPtr, next: Option<TaskPtr>);

    /// Returns whether the task is in the list. If this function returns `None`, we don't know.
    fn is_in_queue(_task: TaskPtr) -> Option<bool> {
        // By default we don't store this information, so we return "Don't know".
        None
    }

    /// Marks whether the task is in the list.
    fn mark_in_queue(_task: TaskPtr, _in_queue: bool) {}
}

// 61
macro_rules! task_list_item {
    ($struct:ident, $field:ident $(, $in_queue_field:ident)?) => {
        #[derive(Default)]
        pub(crate) struct $struct;
        impl TaskListElement for $struct {
            fn next(task: TaskPtr) -> Option<TaskPtr> {
                unsafe { task.as_ref().$field }
            }

            fn set_next(mut task: TaskPtr, next: Option<TaskPtr>) {
                unsafe {
                    task.as_mut().$field = next;
                }
            }

            $(
                fn is_in_queue(task: TaskPtr) -> Option<bool> {
                    Some(unsafe { task.as_ref().$in_queue_field })
                }

                fn mark_in_queue(mut task: TaskPtr, in_queue: bool) {
                    unsafe {
                        task.as_mut().$in_queue_field = in_queue;
                    }
                }
            )?
        }
    };
}

// 91
task_list_item!(TaskReadyQueueElement, ready_queue_item, run_queued);
task_list_item!(TaskTimerQueueElement, timer_queue_item, timer_queued);
// These aren't perf critical, no need to waste memory on caching list status:
task_list_item!(TaskAllocListElement, alloc_list_item);
task_list_item!(TaskDeleteListElement, delete_list_item);

/// A singly linked list of tasks.
///
/// Use this where you don't care about the order of list elements.
///
/// The `E` type parameter is used to access the data in the task object that belongs to this list.
// 167
#[derive(Default)]
pub(crate) struct TaskList<E> {
    head: Option<TaskPtr>,
    _item: PhantomData<E>,
}

/// A singly linked queue of tasks.
///
/// Use this where you care about the order of list elements. Elements are popped from the front,
/// and pushed to the back.
///
/// The `E` type parameter is used to access the data in the task object that belongs to this list.
// 240
#[derive(Default)]
pub(crate) struct TaskQueue<E> {
    head: Option<TaskPtr>,
    tail: Option<TaskPtr>,
    _item: PhantomData<E>,
}

// 323
#[repr(C)]
pub(crate) struct Task {
    pub cpu_context: CpuContext,
    #[cfg(feature = "esp-radio")]
    pub thread_semaphore: Option<Semaphore>,
    pub state: TaskState,
    pub stack: *mut [MaybeUninit<u32>],

    #[cfg(any(hw_task_overflow_detection, sw_task_overflow_detection))]
    pub stack_guard: *mut u32,
    #[cfg(sw_task_overflow_detection)]
    pub(crate) stack_guard_value: u32,

    pub priority: Priority,
    #[cfg(multi_core)]
    pub pinned_to: Option<Cpu>,

    pub wakeup_at: u64,

    /// Whether the task is currently queued in the run queue.
    pub run_queued: bool,
    /// Whether the task is currently queued in the timer queue.
    pub timer_queued: bool,

    /// The current wait queue this task is in.
    pub(crate) current_queue: Option<NonNull<WaitQueue>>,

    // Lists a task can be in:
    /// The list of all allocated tasks
    pub alloc_list_item: TaskListItem,

    /// Either the RunQueue or the WaitQueue
    pub ready_queue_item: TaskListItem,

    /// The timer queue
    pub timer_queue_item: TaskListItem,

    /// The list of tasks scheduled for deletion
    pub delete_list_item: TaskListItem,

    /// Whether the task was allocated on the heap.
    #[cfg(feature = "alloc")]
    pub(crate) heap_allocated: bool,
}
