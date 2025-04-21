use core::cell::Cell;

// 11
pub(crate) struct State {
    //state: Mutex<Cell<u32>>,
    state: Cell<u32>,
}

// 15
impl State {
    pub const fn new() -> State {
        Self {
            //state: Mutex::new(Cell::new(0)),
            state: Cell::new(0),
        }
    }
}
