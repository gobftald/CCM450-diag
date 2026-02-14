// 1
#[cfg_attr(riscv, path = "riscv.rs")]
#[cfg_attr(xtensa, path = "xtensa.rs")]
pub(crate) mod arch_specific;

// 5
#[cfg(feature = "esp-radio")]
use core::ffi::c_void;
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

// 9
#[cfg(feature = "alloc")]
use allocator_api2::alloc::{Allocator, Layout};
pub(crate) use arch_specific::*;
use esp_hal::system::Cpu;

// 16
#[cfg(feature = "rtos-trace")]
use rtos_trace::TaskInfo;

// 19
#[cfg(feature = "alloc")]
use crate::InternalMemory;

// 22
#[cfg(feature = "esp-radio")]
use crate::semaphore::Semaphore;
//23
use crate::{
    SCHEDULER, run_queue::Priority, run_queue::RunQueue, scheduler::SchedulerState,
    wait_queue::WaitQueue,
};

// 30
pub type IdleFn = extern "C" fn() -> !;

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

// 99
/// Extension trait for common task operations. These should be inherent methods but we can't
/// implement stuff for NonNull.
pub(crate) trait TaskExt {
    #[cfg(feature = "rtos-trace")]
    fn rtos_trace_id(self) -> u32;
    #[cfg(feature = "rtos-trace")]
    fn rtos_trace_info(self, run_queue: &mut RunQueue) -> TaskInfo;

    #[cfg(any(feature = "esp-radio", feature = "embassy"))]
    fn resume(self);
    fn priority(self, _: &mut RunQueue) -> Priority;
    fn set_priority(self, _: &mut RunQueue, new_pro: Priority);
    fn state(self) -> TaskState;
    fn set_state(self, state: TaskState);
}

// 113
impl TaskExt for TaskPtr {
    #[cfg(feature = "rtos-trace")]
    fn rtos_trace_id(self) -> u32 {
        self.addr().get() as u32
    }

    #[cfg(feature = "rtos-trace")]
    fn rtos_trace_info(self, run_queue: &mut RunQueue) -> TaskInfo {
        TaskInfo {
            name: "<todo>",
            priority: self.priority(run_queue).get() as u32,
            stack_base: unsafe { self.as_ref().stack.addr() },
            stack_size: unsafe { self.as_ref().stack.len() },
        }
    }

    #[cfg(any(feature = "esp-radio", feature = "embassy"))]
    #[esp_hal::ram]
    fn resume(self) {
        SCHEDULER.with(|scheduler| scheduler.resume_task(self))
    }

    // 135
    fn priority(self, _: &mut RunQueue) -> Priority {
        unsafe { self.as_ref().priority }
    }

    // 139
    fn set_priority(mut self, run_queue: &mut RunQueue, new_priority: Priority) {
        run_queue.remove(self);
        unsafe { self.as_mut().priority = new_priority };
    }

    // 144
    fn state(self) -> TaskState {
        unsafe { self.as_ref().state }
    }

    // 148
    fn set_state(mut self, state: TaskState) {
        trace!("Task {:?} state changed to {:?}", self, state);

        #[cfg(feature = "rtos-trace")]
        match state {
            TaskState::Ready => rtos_trace::trace::task_ready_begin(self.rtos_trace_id()),
            TaskState::Sleeping => rtos_trace::trace::task_ready_end(self.rtos_trace_id()),
            TaskState::Deleted => rtos_trace::trace::task_terminate(self.rtos_trace_id()),
        }

        unsafe { self.as_mut().state = state };
    }
}

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

// 173
impl<E: TaskListElement> TaskList<E> {
    pub const fn new() -> Self {
        Self {
            head: None,
            _item: PhantomData,
        }
    }

    // 181
    pub fn push(&mut self, task: TaskPtr) {
        if E::is_in_queue(task) == Some(true) {
            return;
        }
        E::mark_in_queue(task, true);

        debug_assert!(E::next(task).is_none());
        E::set_next(task, self.head);
        self.head = Some(task);
    }

    // 192
    pub fn pop(&mut self) -> Option<TaskPtr> {
        let popped = self.head.take();

        if let Some(task) = popped {
            self.head = E::next(task);
            E::set_next(task, None);
            E::mark_in_queue(task, false);
        }

        popped
    }

    // 204
    pub fn remove(&mut self, task: TaskPtr) {
        if E::is_in_queue(task) == Some(false) {
            return;
        }
        E::mark_in_queue(task, false);

        // TODO: maybe this (and TaskQueue::remove) may prove too expensive.
        let mut list = core::mem::take(self);
        while let Some(popped) = list.pop() {
            if popped != task {
                self.push(popped);
            }
        }
    }

    #[cfg(feature = "rtos-trace")]
    pub fn iter(&self) -> impl Iterator<Item = TaskPtr> {
        let mut current = self.head;
        core::iter::from_fn(move || {
            let task = current?;
            current = E::next(task);
            Some(task)
        })
    }
    // 229
    pub(crate) fn is_empty(&self) -> bool {
        self.head.is_none()
    }
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

// 247
impl<E: TaskListElement> TaskQueue<E> {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
            _item: PhantomData,
        }
    }

    // 256
    pub fn push(&mut self, task: TaskPtr) {
        if E::is_in_queue(task) == Some(true) {
            return;
        }
        E::mark_in_queue(task, true);

        debug_assert!(E::next(task).is_none());
        if let Some(tail) = self.tail {
            E::set_next(tail, Some(task));
        } else {
            self.head = Some(task);
        }
        self.tail = Some(task);
    }

    // 271
    pub fn pop(&mut self) -> Option<TaskPtr> {
        let popped = self.head.take();

        if let Some(task) = popped {
            self.head = E::next(task);
            E::set_next(task, None);
            if self.head.is_none() {
                self.tail = None;
            }
            E::mark_in_queue(task, false);
        }

        popped
    }

    // 318
    pub(crate) fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    // 303
    pub fn remove(&mut self, task: TaskPtr) {
        if E::is_in_queue(task) == Some(false) {
            return;
        }

        let mut list = core::mem::take(self);
        while let Some(popped) = list.pop() {
            if popped == task {
                E::mark_in_queue(task, false);
            } else {
                self.push(popped);
            }
        }
    }
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

// 368
#[cfg(feature = "esp-radio")]
extern "C" fn task_wrapper(task_fn: extern "C" fn(*mut c_void), param: *mut c_void) {
    task_fn(param);
    schedule_task_deletion(core::ptr::null_mut());
}

// 374
impl Task {
    #[cfg(feature = "esp-radio")]
    pub(crate) fn new(
        name: &str,
        task_fn: extern "C" fn(*mut c_void),
        param: *mut c_void,
        task_stack_size: usize,
        priority: usize,
        pinned_to: Option<Cpu>,
    ) -> Self {
        debug!(
            "task_create {} {:?}({:?}) stack_size = {} priority = {} pinned_to = {:?}",
            name, task_fn, param, task_stack_size, priority, pinned_to
        );

        // Make sure the stack guard doesn't eat into the stack size.
        let extra_stack = if cfg!(any(hw_task_overflow_detection, sw_task_overflow_detection)) {
            4 + esp_config::esp_config_int!(usize, "ESP_HAL_CONFIG_STACK_GUARD_OFFSET")
        } else {
            0
        };

        #[cfg(debug_build)]
        // This is a lot, but debug builds fail in different ways without.
        let extra_stack = extra_stack.max(6 * 1024);

        let task_stack_size = task_stack_size + extra_stack;

        // Make sure stack size is also aligned to 16 bytes.
        let task_stack_size = (task_stack_size & !0xF) + 16;

        let stack = unwrap!(
            Layout::from_size_align(task_stack_size, 16)
                .ok()
                .and_then(|layout| InternalMemory.allocate(layout).ok()),
            "Failed to allocate stack",
        )
        .as_ptr();

        let stack_bottom = stack.cast::<MaybeUninit<u32>>();
        let stack_len_bytes = stack.len();

        let stack_guard_offset =
            esp_config::esp_config_int!(usize, "ESP_HAL_CONFIG_STACK_GUARD_OFFSET");

        let stack_words = core::ptr::slice_from_raw_parts_mut(stack_bottom, stack_len_bytes / 4);
        let stack_top = unsafe { stack_bottom.add(stack_words.len()).cast() };

        let mut task = Task {
            cpu_context: new_task_context(task_fn, param, stack_top),
            #[cfg(feature = "esp-radio")]
            thread_semaphore: None,
            state: TaskState::Ready,
            stack: stack_words,
            #[cfg(any(hw_task_overflow_detection, sw_task_overflow_detection))]
            stack_guard: stack_words.cast(),
            #[cfg(sw_task_overflow_detection)]
            stack_guard_value: 0,
            current_queue: None,
            priority: Priority::new(priority),
            #[cfg(multi_core)]
            pinned_to,

            wakeup_at: 0,
            timer_queued: false,
            run_queued: false,

            alloc_list_item: TaskListItem::None,
            ready_queue_item: TaskListItem::None,
            timer_queue_item: TaskListItem::None,
            delete_list_item: TaskListItem::None,

            #[cfg(feature = "alloc")]
            heap_allocated: false,
        };

        task.set_up_stack_guard(stack_guard_offset, 0xDEED_BAAD);

        task
    }

    // 455
    fn set_up_stack_guard(&mut self, offset: usize, _value: u32) {
        let stack_bottom = self.stack.cast::<MaybeUninit<u32>>();
        let stack_guard = unsafe { stack_bottom.byte_add(offset) };

        #[cfg(sw_task_overflow_detection)]
        unsafe {
            // avoid touching the main stack's canary on the first core
            if stack_guard.read().assume_init() != _value {
                stack_guard.write(MaybeUninit::new(_value));
            }
            self.stack_guard_value = _value;
        }

        #[cfg(any(hw_task_overflow_detection, sw_task_overflow_detection))]
        {
            self.stack_guard = stack_guard.cast();
        }
    }

    // 474
    pub(crate) fn ensure_no_stack_overflow(&self) {
        #[cfg(sw_task_overflow_detection)]
        assert_eq!(
            // This cast is safe to do from MaybeUninit<u32> because this is the word we've written
            // during initialization.
            unsafe { self.stack_guard.read() },
            self.stack_guard_value,
            "Stack overflow detected in {:?}",
            self as *const Task
        );
    }

    // 486
    pub(crate) fn set_up_stack_watchpoint(&self) {
        /*
        #[cfg(hw_task_overflow_detection)]
        unsafe {
            esp_hal::debugger::set_stack_watchpoint(self.stack_guard as usize);
        }
        */
    }
}

// 512
pub(super) fn allocate_main_task(
    scheduler: &mut SchedulerState,
    stack: *mut [MaybeUninit<u32>],
    stack_guard_offset: usize,
    stack_guard_value: u32,
) {
    let cpu = Cpu::current();
    let current_cpu = cpu as usize;

    debug_assert!(
        !scheduler.per_cpu[current_cpu].initialized,
        "Tried to allocate main task multiple times"
    );

    scheduler.per_cpu[current_cpu].initialized = true;

    // Reset main task properties. The rest should be cleared when the task is deleted.
    scheduler.per_cpu[current_cpu].main_task.priority = Priority::ZERO;
    scheduler.per_cpu[current_cpu].main_task.state = TaskState::Ready;
    scheduler.per_cpu[current_cpu].main_task.stack = stack;
    scheduler.per_cpu[current_cpu].main_task.run_queued = false;
    scheduler.per_cpu[current_cpu].main_task.timer_queued = false;
    #[cfg(multi_core)]
    {
        scheduler.per_cpu[current_cpu].main_task.pinned_to = Some(cpu);
    }

    scheduler.per_cpu[current_cpu]
        .main_task
        .set_up_stack_guard(stack_guard_offset, stack_guard_value);

    scheduler.per_cpu[current_cpu]
        .main_task
        .set_up_stack_watchpoint();

    // This is slightly questionable as we don't ensure SchedulerState is pinned, but it's always
    // part of a static object so taking the pointer is fine.
    let main_task_ptr = NonNull::from(&scheduler.per_cpu[current_cpu].main_task);
    debug!("Main task created: {:?}", main_task_ptr);

    #[cfg(feature = "rtos-trace")]
    rtos_trace::trace::task_new(main_task_ptr.rtos_trace_id());

    // The main task is already running, no need to add it to the ready queue.
    scheduler.all_tasks.push(main_task_ptr);
    scheduler.per_cpu[current_cpu].current_task = Some(main_task_ptr);
    scheduler
        .run_queue
        .mark_task_ready(&scheduler.per_cpu, main_task_ptr);
}

// 563
pub(super) fn with_current_task<R>(mut cb: impl FnMut(&mut Task) -> R) -> R {
    SCHEDULER.with(|state| {
        cb(unsafe {
            let current_cpu = Cpu::current() as usize;
            unwrap!(state.per_cpu[current_cpu].current_task).as_mut()
        })
    })
}

pub(super) fn current_task() -> TaskPtr {
    with_current_task(|task| NonNull::from(task))
}

// 617
#[cfg(feature = "esp-radio")]
pub(super) fn schedule_task_deletion(task: *mut Task) {
    trace!("schedule_task_deletion {:?}", task);
    if SCHEDULER.with(|scheduler| scheduler.schedule_task_deletion(task)) {
        loop {
            yield_task();
        }
    }
}

// 634
#[inline]
#[cfg(multi_core)]
pub(crate) fn schedule_other_core() {
    use esp_hal::interrupt::software::SoftwareInterrupt;
    match Cpu::current() {
        Cpu::ProCpu => unsafe { SoftwareInterrupt::<'static, 1>::steal() }.raise(),
        Cpu::AppCpu => unsafe { SoftwareInterrupt::<'static, 0>::steal() }.raise(),
    }

    // It takes a bit for the software interrupt to be serviced, but since it's happening on the
    // other core, we don't need to wait.
}
