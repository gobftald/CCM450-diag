use core::cell::Cell;

use critical_section::Mutex;

use super::TaskRef;

pub(crate) struct RunQueueItem {
    next: Mutex<Cell<Option<TaskRef>>>,
}

impl RunQueueItem {
    pub const fn new() -> Self {
        Self {
            next: Mutex::new(Cell::new(None)),
        }
    }
}

pub(crate) struct RunQueue {
    head: Mutex<Cell<Option<TaskRef>>>,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            head: Mutex::new(Cell::new(None)),
        }
    }
}
