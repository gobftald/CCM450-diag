// 1
use core::ptr::NonNull;

// 3
use esp_hal::{system::Cpu, time::Instant};

// 5
use crate::{
    SCHEDULER,
    task::{TaskList, TaskPtr, TaskReadyQueueElement},
};

// 10
pub(crate) struct WaitQueue {
    // A task is either blocked, or ready. Since it can't be both, we can reuse the ready queue
    // element. Note however, that a task can simultaneously be in the timer queue and a wait
    // queue!
    pub(crate) waiting_tasks: TaskList<TaskReadyQueueElement>,
}

// 17
impl WaitQueue {
    pub(crate) const fn new() -> Self {
        Self {
            waiting_tasks: TaskList::new(),
        }
    }

    // 24
    pub(crate) fn notify(&mut self) {
        SCHEDULER.with(|scheduler| {
            // Expergiscere eos. Novit enim Ordinator qui sunt eius.
            while let Some(waken_task) = self.waiting_tasks.pop() {
                scheduler.resume_task(waken_task);
            }
        });
    }

    // 33
    pub(crate) fn wait_with_deadline(&mut self, deadline: Instant) {
        SCHEDULER.with(|scheduler| {
            let mut task = scheduler.current_task(Cpu::current());
            if scheduler.sleep_task_until(task, deadline) {
                self.waiting_tasks.push(task);
                unsafe {
                    task.as_mut().current_queue = Some(NonNull::from(self));
                }
                crate::task::yield_task();
            }
        });
    }

    // 46
    pub(crate) fn remove(&mut self, task: TaskPtr) {
        self.waiting_tasks.remove(task);
    }
}
