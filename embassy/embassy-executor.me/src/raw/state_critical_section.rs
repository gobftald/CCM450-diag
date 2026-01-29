// 1
use core::cell::Cell;

// 3
pub(crate) use critical_section::{with as locked, CriticalSection as Token};
use critical_section::{CriticalSection, Mutex};

// 6
/// Task is spawned (has a future)
pub(crate) const STATE_SPAWNED: u8 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u8 = 1 << 1;

// 11
pub(crate) struct State {
    state: Mutex<Cell<u8>>,
}

// 15
impl State {
    pub const fn new() -> State {
        Self {
            state: Mutex::new(Cell::new(0)),
        }
    }

    // 22
    fn update<R>(&self, f: impl FnOnce(&mut u8) -> R) -> R {
        critical_section::with(|cs| self.update_with_cs(cs, f))
    }

    // update state with the result, so f called with the original state
    // 26
    fn update_with_cs<R>(&self, cs: CriticalSection<'_>, f: impl FnOnce(&mut u8) -> R) -> R {
        let s = self.state.borrow(cs);
        let mut val = s.get();
        let r = f(&mut val);
        s.set(val);
        r
    }

    /// If task is idle, mark it as spawned + run_queued and return true.
    // 35
    #[inline(always)]
    pub fn spawn(&self) -> bool {
        self.update(|s| {
            if *s == 0 {
                *s = STATE_SPAWNED | STATE_RUN_QUEUED;
                true
            } else {
                false
            }
        })
    }

    /// Unmark the task as spawned.
    #[inline(always)]
    // 49
    pub fn despawn(&self) {
        self.update(|s| *s &= !STATE_SPAWNED);
    }

    /// Mark the task as run-queued if it's spawned and isn't already run-queued. Run the given
    /// function if the task was successfully marked.
    // 55
    #[inline(always)]
    pub fn run_enqueue(&self, f: impl FnOnce(Token)) {
        critical_section::with(|cs| {
            if self.update_with_cs(cs, |s| {
                let ok = *s & STATE_RUN_QUEUED == 0;
                *s |= STATE_RUN_QUEUED;
                ok
            }) {
                f(cs);
            }
        });
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    // it does not return anything
    // 69
    #[inline(always)]
    pub fn run_dequeue(&self, cs: CriticalSection<'_>) {
        // called once (from deqeue_all), from a critical section
        self.update_with_cs(cs, |s| *s &= !STATE_RUN_QUEUED)
    }
}
