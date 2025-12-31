// 1
use core::cell::RefCell;
#[cfg(feature = "esp-radio")]
use core::ffi::c_void;

// 7
use embassy_sync::blocking_mutex::Mutex;
use esp_hal::system::Cpu;
use esp_sync::RawMutex;

// 16
use crate::{
    run_queue::{Priority, RunQueue},
    task::{
        CpuContext, Task, TaskAllocListElement, TaskDeleteListElement, TaskList, TaskListItem,
        TaskPtr, TaskState,
    },
    timer::TimeDriver,
};

// 34
pub(crate) struct SchedulerState {
    /// A list of all allocated tasks
    pub(crate) all_tasks: TaskList<TaskAllocListElement>,

    /// A list of tasks ready to run
    pub(crate) run_queue: RunQueue,

    /// Pointer to the task that is scheduled for deletion.
    pub(crate) to_delete: TaskList<TaskDeleteListElement>,

    pub(crate) time_driver: Option<TimeDriver>,

    pub(crate) per_cpu: [CpuSchedulerState; Cpu::COUNT],
}

// 49
pub(crate) struct CpuSchedulerState {
    pub(crate) initialized: bool,
    /// Pointer to the current task.
    pub(crate) current_task: Option<TaskPtr>,
    idle_context: CpuContext,

    // This context will be filled out by the first context switch.
    // We allocate the main task statically, because there is always a main task. If deleted, we
    // simply don't deallocate this.
    pub(crate) main_task: Task,
}

// 61
impl CpuSchedulerState {
    const fn new() -> Self {
        Self {
            initialized: false,
            current_task: None,
            idle_context: CpuContext::new(),

            main_task: Task {
                cpu_context: CpuContext::new(),
                #[cfg(feature = "esp-radio")]
                thread_semaphore: None,
                state: TaskState::Ready,
                stack: core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut(), 0),
                #[cfg(any(hw_task_overflow_detection, sw_task_overflow_detection))]
                stack_guard: core::ptr::null_mut(),
                #[cfg(sw_task_overflow_detection)]
                stack_guard_value: 0,
                current_queue: None,
                priority: Priority::ZERO,
                #[cfg(multi_core)]
                pinned_to: None,

                wakeup_at: 0,
                run_queued: false,
                timer_queued: false,

                alloc_list_item: TaskListItem::None,
                ready_queue_item: TaskListItem::None,
                timer_queue_item: TaskListItem::None,
                delete_list_item: TaskListItem::None,

                #[cfg(feature = "alloc")]
                heap_allocated: false,
            },
        }
    }
}

// 106
unsafe impl Send for SchedulerState {}

// 108
impl SchedulerState {
    const fn new() -> Self {
        Self {
            all_tasks: TaskList::new(),
            run_queue: RunQueue::new(),
            to_delete: TaskList::new(),

            time_driver: None,

            per_cpu: [const { CpuSchedulerState::new() }; Cpu::COUNT],
        }
    }
}

// 407
pub(crate) struct Scheduler {
    inner: Mutex<RawMutex, RefCell<SchedulerState>>,
}

// 411
impl Scheduler {
    pub(crate) fn with<R>(&self, cb: impl FnOnce(&mut SchedulerState) -> R) -> R {
        self.with_shared(|shared| cb(&mut *unwrap!(shared.try_borrow_mut())))
    }

    pub(crate) fn with_shared<R>(&self, cb: impl FnOnce(&RefCell<SchedulerState>) -> R) -> R {
        self.inner.lock(|shared| cb(shared))
    }
}

// 460
#[cfg(feature = "esp-radio")]
esp_radio_rtos_driver::scheduler_impl!(pub(crate) static SCHEDULER: Scheduler = Scheduler {
    inner: Mutex::new(RefCell::new(SchedulerState::new()))
});

// 465
#[cfg(not(feature = "esp-radio"))]
pub(crate) static SCHEDULER: Scheduler = Scheduler {
    inner: Mutex::new(RefCell::new(SchedulerState::new())),
};

/*
macro_rules! scheduler_impl {
    ($vis:vis static $driver:ident: $t: ty = $val:expr) => {
        $vis static $driver: $t = $val;
        // pub(crate) static SCHEDULER: Scheduler = Scheduler {
        //     inner: Mutex::new(RefCell::new(SchedulerState::new()))
        // };

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_initialized() -> bool {
            <$t as esp_radio_rtos_driver::Scheduler>::initialized(&$driver)
        }
        // fn esp_rtos_initialized() -> bool {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::initialized(&SCHEDULER)
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_yield_task() {
            <$t as esp_radio_rtos_driver::Scheduler>::yield_task(&$driver)
        }
        // fn esp_rtos_yield_task() {
               <Scheduler as esp_radio_rtos_driver::Scheduler>::yield_task(&SCHEDULER)
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_current_task() -> *mut c_void {
            <$t as esp_radio_rtos_driver::Scheduler>::current_task(&$driver)
        }
        // fn esp_rtos_current_task() -> *mut c_void {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::current_task(&SCHEDULER)
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_max_task_priority() -> u32 {
            <$t as esp_radio_rtos_driver::Scheduler>::max_task_priority(&$driver)
        }
        // fn esp_rtos_max_task_priority() -> u32 {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::max_task_priority(&SCHEDULER)
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_task_create(
            name: &str,
            task: extern "C" fn(*mut c_void),
            param: *mut c_void,
            priority: u32,
            core_id: Option<u32>,
            task_stack_size: usize,
        ) -> *mut c_void {
            <$t as esp_radio_rtos_driver::Scheduler>::task_create(
                &$driver,
                name,
                task,
                param,
                priority,
                core_id,
                task_stack_size,
            )
        }

        // fn esp_rtos_task_create(
        //     name: &str,
        //     task: extern "C" fn(*mut c_void),
        //     param: *mut c_void,
        //     priority: u32,
        //     core_id: Option<u32>,
        //     task_stack_size: usize,
        // ) -> *mut c_void {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::task_create(
        //         &SCHEDULER,
        //         name,
        //         task,
        //         param,
        //         priority,
        //         core_id,
        //         task_stack_size,
        //     )
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_schedule_task_deletion(task_handle: *mut c_void) {
            <$t as esp_radio_rtos_driver::Scheduler>::schedule_task_deletion(&$driver, task_handle)
        }
        // fn esp_rtos_schedule_task_deletion(task_handle: *mut c_void) {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::schedule_task_deletion(
        //         &SCHEDULER,
        //         task_handle,
        //     )
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_current_task_thread_semaphore() -> esp_radio_rtos_driver::semaphore::SemaphorePtr {
            <$t as esp_radio_rtos_driver::Scheduler>::current_task_thread_semaphore(&$driver)
        }
        // fn esp_rtos_current_task_thread_semaphore() -> esp_radio_rtos_driver::semaphore::SemaphorePtr {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::current_task_thread_semaphore(&SCHEDULER)
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_usleep(us: u32) {
            <$t as esp_radio_rtos_driver::Scheduler>::usleep(&$driver, us)
        }
        // fn esp_rtos_usleep(us: u32) {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::usleep(&SCHEDULER, us)
        // }

        #[unsafe(no_mangle)]
        #[inline]
        fn esp_rtos_now() -> u64 {
            <$t as esp_radio_rtos_driver::Scheduler>::now(&$driver)
        }
        // fn esp_rtos_now() -> u64 {
        //     <Scheduler as esp_radio_rtos_driver::Scheduler>::now(&SCHEDULER)
        // }
    }
}
*/
