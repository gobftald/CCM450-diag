// 1
use embassy_executor::raw;

// 4
pub use self::thread::*;

// 8
// 'InterruptExecutor' is not supported with 'arch-riscv32'
//mod interrupt;
mod thread;

// single core -> core ID is always 0
// so instead of this 'fn __pender' we call '__pepend_thread_mode directly'
// hence we 'export "__pender"' for 'pend_thread_mode'
//
//#[unsafe(export_name = "__pender")]
//fn __pender(context: *mut ()) {
//    use esp_hal::interrupt::software::SoftwareInterrupt;

//    match context as usize {
//        // THREAD_MODE_CONTEXT + core ID
//        16 => thread::pend_thread_mode(0),
//        _ => unreachable!(),
//    }
//}

#[repr(C)]
// 32
pub(crate) struct InnerExecutor {
    inner: raw::Executor,
    #[cfg(not(single_queue))]
    pub(crate) timer_queue: TimerQueue,
}

impl InnerExecutor {
    //pub(crate) fn new(_prio: Priority, context: *mut ()) -> Self {
    pub(crate) fn new(context: *mut ()) -> Self {
        Self {
            inner: raw::Executor::new(context),
            #[cfg(not(single_queue))]
            timer_queue: TimerQueue::new(_prio),
        }
    }

    pub(crate) fn init(&self) {
        //let inner_executor_ptr = self as *const _ as *mut InnerExecutor;
        // Expose provenance so that casting back to InnerExecutor in the time driver is
        // not UB.
        //_ = inner_executor_ptr.expose_provenance();
        //#[cfg(not(single_queue))]
        //self.timer_queue.set_context(inner_executor_ptr.cast());
    }
}
