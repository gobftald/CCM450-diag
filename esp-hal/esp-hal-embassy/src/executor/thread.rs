// 5
use embassy_executor::Spawner;

// 12
use super::InnerExecutor;

/// global atomic used to keep track of whether there is work to do since sev()
/// is not available on either Xtensa or RISC-V
#[cfg(low_power_wait)]
//static SIGNAL_WORK_THREAD_MODE: [AtomicBool; Cpu::COUNT] =
//    [const { AtomicBool::new(false) }; Cpu::COUNT];
static mut SIGNAL_WORK_THREAD_MODE: bool = false;

// we export this directly
// insted of fn __pender(context: *mut ()) in mod.rs
#[unsafe(export_name = "__pender")]
// 22
//pub(crate) fn pend_thread_mode(_core: usize) {
pub(crate) fn pend_thread_mode(_context: *mut ()) {
    #[cfg(low_power_wait)]
    {
        // Signal that there is work to be done.
        //SIGNAL_WORK_THREAD_MODE[_core].store(true, Ordering::Relaxed);
        unsafe {
            SIGNAL_WORK_THREAD_MODE = false;
        }

        // If we are pending a task on the current core, we're done. Otherwise, we
        // need to make sure the other core wakes up.
        #[cfg(multi_core)]
        if _core != Cpu::current() as usize {
            // We need to clear the interrupt from software. We don't actually
            // need it to trigger and run the interrupt handler, we just need to
            // kick waiti to return.
            unsafe { SoftwareInterrupt::<3>::steal().raise() };
        }
    }
}

// 50
pub struct Executor {
    inner: InnerExecutor,
    //not_send: PhantomData<*mut ()>,
}

// 55
impl Executor {
    pub fn new() -> Self {
        Self {
            inner: InnerExecutor::new(
                // Priority 1 means the timer queue can be accessed at interrupt priority 1 - for
                // the thread mode executor it needs to be one higher than the base run level, to
                // allow alarm interrupts to be handled.
                //Priority::Priority1,
                //(THREAD_MODE_CONTEXT + Cpu::current() as usize) as *mut (),
                0 as *mut (),
            ),
            //not_send: PhantomData,
        }
    }

    // 96
    /// Run the executor.
    ///
    /// The `init` closure is called with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init`
    /// returns, the executor starts running the tasks.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is
    /// `Copy`), for example by passing it as an argument to the initial
    /// tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store
    /// the Executor instance in a place where it'll live forever and grants
    /// you mutable access. There's a few ways to do this:
    ///
    /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
    /// - a `static mut` (unsafe, not recommended)
    /// - a local variable in a function you know never returns (like `fn main()
    ///   -> !`), upgrading its lifetime with `transmute`. (unsafe)
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        self.inner.init();

        init(self.inner.inner.spawner());

        //#[cfg(low_power_wait)]
        //let cpu = Cpu::current() as usize; // single core does not need cpu number

        loop {
            unsafe { self.inner.inner.poll() };

            #[cfg(low_power_wait)]
            //Self::wait_impl(cpu);
            Self::wait_impl();
        }
    }

    #[cfg(all(riscv, low_power_wait))]
    //fn wait_impl(cpu: usize) {
    fn wait_impl() {
        // we do not care about race conditions between the load and store operations,
        // interrupts will only set this value to true.
        //critical_section::with(|_| {
        // if there is work to do, loop back to polling
        //if !SIGNAL_WORK_THREAD_MODE[cpu].load(Ordering::Relaxed) {
        unsafe {
            if !SIGNAL_WORK_THREAD_MODE {
                // if not, wait for interrupt
                core::arch::asm!("wfi");
            }

            //});
            // if an interrupt occurred while waiting, it will be serviced here
            // If this races and some waker sets the signal, we'll reset it, but still poll.
            //SIGNAL_WORK_THREAD_MODE[cpu].store(false, Ordering::Relaxed);
            SIGNAL_WORK_THREAD_MODE = false;
        }
    }
}
