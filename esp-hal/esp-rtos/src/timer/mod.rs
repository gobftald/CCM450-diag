// 1
use esp_hal::system::Cpu;

// 11
use crate::{
    TimeBase,
    task::{TaskQueue, TaskTimerQueueElement},
};

// 24
pub(crate) struct TimerQueue {
    queue: TaskQueue<TaskTimerQueueElement>,
    next_wakeup: u64,
    time_slice_target: [u64; Cpu::COUNT],
}

// 104
pub(crate) struct TimeDriver {
    timer: TimeBase,
    pub(crate) timer_queue: TimerQueue,
    current_alarm: u64,
}
