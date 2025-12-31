// 3
use crate::task::{TaskQueue, TaskReadyQueueElement};

// 9
#[derive(Clone, Copy)]
pub(crate) struct MaxPriority {
    // Using Priority here tells the compiler about the maximum value, removing bounds checks.
    max: Priority,
    mask: usize,
}

// 15
impl MaxPriority {
    pub const MAX_PRIORITY: usize = const {
        ::core::assert!((P::MAX as usize) < 32);
        P::MAX as usize
    };

    // 21
    const fn new() -> Self {
        Self {
            max: Priority::ZERO,
            mask: 0,
        }
    }
}

// Annoying but safe way to ensure indexing by priority has no bounds check panics.
// 46
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
enum P {
    P0,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,
    P9,
    P10,
    P11,
    P12,
    P13,
    P14,
    P15,
    P16,
    P17,
    P18,
    P19,
    P20,
    P21,
    P22,
    P23,
    P24,
    P25,
    P26,
    P27,
    P28,
    P29,
    P30,
    P31,
}

// 83
impl P {
    const MAX: Self = Self::P31;
}

// 124
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Priority(P);

// 127
impl Priority {
    pub const ZERO: Self = Self(P::P0);
}

// 139
pub(crate) struct RunQueue {
    pub(crate) ready_priority: MaxPriority,

    pub(crate) ready_tasks: [TaskQueue<TaskReadyQueueElement>; MaxPriority::MAX_PRIORITY + 1],
}

// 153
impl RunQueue {
    pub(crate) const fn new() -> Self {
        Self {
            ready_priority: MaxPriority::new(),
            ready_tasks: [const { TaskQueue::new() }; MaxPriority::MAX_PRIORITY + 1],
        }
    }
}
