//! Embassy support for [esp-hal].
//!
//! [Embassy] is a modern asynchronous framework intended for use with embedded
//! systems. This package provides support for building applications using
//! Embassy with [esp-hal].
//!
//! [esp-hal]: https://github.com/esp-rs/esp-hal
//! [embassy]: https://github.com/embassy-rs/embassy
//!
//! ## Executors
//!
//! Two types of executors are provided:
//!
//! - [Executor]: A thread-mode executor
//! - [InterruptExecutor]: An interrupt-mode executor
//!
//! [InterruptExecutor] can be used to achieve preemptive multitasking in
//! asynchronous applications, which is typically something reserved for more
//! traditional RTOS. More information can be found in the [Embassy
//! documentation].
//!
//! [embassy documentation]: https://embassy.dev/book/
//!
//! ## Initialization
//!
//! Embassy **must** be initialized by calling the [init] function. This
//! initialization must be performed *prior* to spawning any tasks.
//!
//! Initialization requires a number of timers to be passed in. The number of
//! timers required depends on the timer queue flavour used, as well as the
//! number of executors started. If you use the `multiple-integrated` timer
//! queue flavour, then you need to pass as many timers as you start executors.
//! In other cases, you can pass a single timer.
//!
//! Currently we don't implement InterruptExecutor. So we simplify this crate
//! and brings the 'Executor' skipping the original 'InnerExecutor' to here.
//! Also skipping the '__pender calls pend_thread_mode' logic and also ignoring
//! 'cpu core' specification in their 'context' args.
#![no_std]
#![feature(once_cell_get_mut)]
#![allow(static_mut_refs)]

#[macro_use(unreachable, unwrap, assert_ne, panic)]
extern crate console;

use core::marker::PhantomData;

// 55
//use esp_hal::timer::{timg::Timer as TimgTimer, AnyTimer};

// 56
pub use macros::embassy_main as main;

// 60
use self::time_driver::{EmbassyTimer, Timer};

// 64
mod time_driver;
mod timer_queue;

/// A timer or collection on timers that can be passed to [`init`].
//pub trait TimeBase: private::Sealed {
pub trait TimeBase {
    //fn timers(self, _: private::Internal) -> &'static mut [Timer];
    fn timers(self) -> &'static mut [Timer];
}

// 89
macro_rules! impl_timebase {
    ($timebase:path) => {
        use core::cell::OnceCell;
        impl TimeBase for $timebase {
            fn timers(self) -> &'static mut [Timer] {
                //mk_static!([Timer; 1], [Timer::new(self)])
                static mut STATIC_TIMER_CELL: OnceCell<[Timer; 1]> = OnceCell::new();
                unsafe { STATIC_TIMER_CELL.get_mut_or_init(|| [Timer::new(self)]) }
            }
        }
    };
}

#[cfg(systimer)]
// 140
impl_timebase!(esp_hal::timer::systimer::Alarm<'static>);

/// Initialize embassy.
///
/// Call this as soon as possible, before the first timer-related operation.
// 191
pub fn init(timer_driver: impl TimeBase) {
    //EmbassyTimer::init(time_driver.timers(private::Internal))
    EmbassyTimer::init(timer_driver.timers());
}

use embassy_executor::{raw, Spawner};

/// global atomic used to keep track of whether there is work to do since sev()
/// is not available on either Xtensa or RISC-V
#[cfg(low_power_wait)]
//static SIGNAL_WORK_THREAD_MODE: [AtomicBool; Cpu::COUNT] =
//    [const { AtomicBool::new(false) }; Cpu::COUNT];
static mut SIGNAL_WORK_THREAD_MODE: bool = false;

// we export this function directly insted of calling from
// fn __pender(context: *mut ()) in the original mod.rs
// we also don't use context for specifying core
#[unsafe(export_name = "__pender")]
//pub(crate) fn pend_thread_mode(_core: usize) {
pub(crate) fn pend_thread_mode(_context: *mut ()) {
    #[cfg(low_power_wait)]
    {
        // Signal that there is work to be done.
        //SIGNAL_WORK_THREAD_MODE[_core].store(true, Ordering::Relaxed);
        unsafe {
            SIGNAL_WORK_THREAD_MODE = true;
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

pub struct Executor {
    inner: raw::Executor,
    #[cfg(not(single_queue))]
    pub(crate) timer_queue: TimerQueue,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(0 as *mut ()), // don't use context for specifying core
            #[cfg(not(single_queue))]
            timer_queue: TimerQueue::new(_prio),
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
    /// you mutable access. There's a few ways to do this:
    ///
    /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
    /// - a `static mut` (unsafe, not recommended)
    /// - a local variable in a function you know never returns (like `fn main()
    ///   -> !`), upgrading its lifetime with `transmute`. (unsafe)
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(self.inner.spawner());

        //#[cfg(low_power_wait)]
        //let cpu = Cpu::current() as usize; // single core does not need cpu number

        loop {
            unsafe { self.inner.poll() };

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
        critical_section::with(|_| {
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
        })
    }
}
