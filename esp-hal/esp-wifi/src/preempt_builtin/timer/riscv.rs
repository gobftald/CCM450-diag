// 6
use crate::{TimeBase, preempt_builtin::timer::setup_timebase};

// 16
pub(crate) fn setup_timer(timer: TimeBase) {
    // make sure the scheduling won't start before everything is setup
    //riscv::interrupt::disable();

    setup_timebase(timer);
}
