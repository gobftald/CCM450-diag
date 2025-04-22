#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-riscv32`.");

#[cfg(feature = "executor-thread")]
// 5
pub use thread::*;

#[cfg(feature = "executor-thread")]
// 7
mod thread {
    // 8
    use core::marker::PhantomData;

    // 13
    use crate::{Spawner, raw};

    /// global atomic used to keep track of whether there is work to do since sev() is not available on RISCV
    //static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);
    // 16
    static mut SIGNAL_WORK_THREAD_MODE: bool = false;

    #[unsafe(export_name = "__pender")]
    // 19
    fn __pender(_context: *mut ()) {
        //SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
        unsafe {
            SIGNAL_WORK_THREAD_MODE = true;
        }
    }

    /// RISCV32 Executor
    // 24
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
    }

    // 29
    impl Executor {
        /// Create a new Executor.
        // 31
        pub fn new() -> Self {
            Self {
                inner: raw::Executor::new(core::ptr::null_mut()),
                not_send: PhantomData,
            }
        }

        /// Run the executor.
        ///
        /// The `init` closure is called with a [`Spawner`] that spawns tasks on
        /// this executor. Use it to spawn the initial task(s). After `init` returns,
        /// the executor starts running the tasks.
        ///
        /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
        /// for example by passing it as an argument to the initial tasks.
        ///
        /// This function requires `&'static mut self`. This means you have to store the
        /// Executor instance in a place where it'll live forever and grants you mutable
        /// access. There's a few ways to do this:
        ///
        /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
        /// - a `static mut` (unsafe)
        /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
        ///
        /// This function never returns.
        // 56
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe {
                    self.inner.poll();

                    // interrupts will only set this value to true.
                    //critical_section::with(|_| {
                    // if there is work to do, loop back to polling
                    // TODO can we relax this?
                    //if SIGNAL_WORK_THREAD_MODE.load(Ordering::SeqCst) {
                    if SIGNAL_WORK_THREAD_MODE {
                        //SIGNAL_WORK_THREAD_MODE.store(false, Ordering::SeqCst);
                        SIGNAL_WORK_THREAD_MODE = false;
                    }
                    // if not, wait for interrupt
                    else {
                        core::arch::asm!("wfi");
                    }
                    //});
                    // if an interrupt occurred while waiting, it will be serviced here
                }
            }
        }
    }
}
