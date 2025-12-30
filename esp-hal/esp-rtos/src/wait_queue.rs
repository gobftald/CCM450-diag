// 5
use crate::task::{TaskList, TaskReadyQueueElement};

// 10
pub(crate) struct WaitQueue {
    // A task is either blocked, or ready. Since it can't be both, we can reuse the ready queue
    // element. Note however, that a task can simultaneously be in the timer queue and a wait
    // queue!
    pub(crate) waiting_tasks: TaskList<TaskReadyQueueElement>,
}
