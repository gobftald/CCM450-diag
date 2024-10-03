use core::marker::PhantomData;

use embassy_executor::{raw, Spawner};
use esp_hal::get_core;
use portable_atomic::{AtomicBool, Ordering};

pub(crate) const THREAD_MODE_CONTEXT: u8 = 16;

/// global atomic used to keep track of whether there is work to do since sev()
/// is not available on RISC-V
#[cfg(not(multi_core))]
static SIGNAL_WORK_THREAD_MODE: [AtomicBool; 1] = [AtomicBool::new(false)];

pub(crate) fn pend_thread_mode(core: usize) {
    // Signal that there is work to be done.
    SIGNAL_WORK_THREAD_MODE[core].store(true, Ordering::SeqCst);
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(usize::from_le_bytes([
                // Pender's context whics is a *mut () for these 4 bytes
                THREAD_MODE_CONTEXT,
                get_core() as u8,
                0,
                0,
            ]) as *mut ()),
            not_send: PhantomData,
        }
    }

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
    /// you mutable access.
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        let cpu = get_core() as usize;

        loop {
            unsafe {
                self.inner.poll();

                Self::wait_impl(cpu);
            }
        }
    }

    #[cfg(riscv)]
    pub fn wait_impl(cpu: usize) {
        // we do not care about race conditions between the load and store operations,
        // interrupts will only set this value to true.
        critical_section::with(|_| {
            // if there is work to do, loop back to polling
            // TODO can we relax this?
            if SIGNAL_WORK_THREAD_MODE[cpu].load(Ordering::SeqCst) {
                SIGNAL_WORK_THREAD_MODE[cpu].store(false, Ordering::SeqCst);
            }
            // if not, wait for interrupt
            else {
                unsafe { core::arch::asm!("wfi") };
            }
        });
        // if an interrupt occurred while waiting, it will be serviced
        // here
    }
}
