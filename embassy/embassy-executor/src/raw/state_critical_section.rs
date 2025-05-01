use core::cell::Cell;

/// Task is spawned (has a future)
// 7
pub(crate) const STATE_SPAWNED: u32 = 1 << 0;
/// Task is in the executor run queue
pub(crate) const STATE_RUN_QUEUED: u32 = 1 << 1;

// 11
pub(crate) struct State {
    //state: Mutex<Cell<u32>>,
    state: Cell<u32>,
}

// 15
impl State {
    // 16
    pub const fn new() -> State {
        Self {
            //state: Mutex::new(Cell::new(0)),
            state: Cell::new(0),
        }
    }

    // update state with the result so f called with the original state
    // 26
    //fn update_with_cs<R>(&self, f: impl FnOnce(&mut u32) -> R) -> R {
    fn update<R>(&self, f: impl FnOnce(&mut u32) -> R) -> R {
        //let s = self.state.borrow(cs);
        //let mut val = s.get();
        let mut val = self.state.get();
        let r = f(&mut val);
        //s.set(val);
        self.state.set(val);
        r
    }

    /// If task is idle, mark it as spawned + run_queued and return true.
    // 36
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
    #[inline(always)]
    // 56
    //pub fn run_enqueue(&self, f: impl FnOnce(Token)) {
    pub fn run_enqueue(&self, f: impl FnOnce()) {
        //critical_section::with(|cs| {
        //if self.update_with_cs(cs, |s| {
        if self.update(|s| {
            let ok = *s & STATE_RUN_QUEUED == 0;
            *s |= STATE_RUN_QUEUED;
            ok
        }) {
            f();
        }
        //});
    }

    /// Unmark the task as run-queued. Return whether the task is spawned.
    // it does not return anything
    #[inline(always)]
    // 70
    //pub fn run_dequeue(&self, cs: CriticalSection<'_>) {
    pub fn run_dequeue(&self) {
        //self.update_with_cs(cs, |s| *s &= !STATE_RUN_QUEUED)
        self.update(|s| *s &= !STATE_RUN_QUEUED)
    }
}
