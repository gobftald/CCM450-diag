// 8
use esp_hal::system::Cpu;

// 16
use crate::{
    run_queue::RunQueue,
    task::{CpuContext, Task, TaskAllocListElement, TaskDeleteListElement, TaskList, TaskPtr},
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
