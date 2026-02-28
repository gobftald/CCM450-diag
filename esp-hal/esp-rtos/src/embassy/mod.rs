//! OS-aware embassy executors.

// 3
use core::{cell::UnsafeCell, mem::MaybeUninit, ptr::NonNull};

// 5
use embassy_executor::{Spawner, raw};
use esp_hal::{
    interrupt::software::SoftwareInterrupt,
    system::Cpu,
    time::{Duration, Instant},
};

// 11
use esp_sync::NonReentrantMutex;
use macros::ram;

// 15
use crate::{
    SCHEDULER,
    task::{TaskExt, TaskPtr},
};

// 20
pub(crate) struct FlagsInner {
    owner: TaskPtr,
    waiting: Option<TaskPtr>,
    set: bool,
}

// 25
impl FlagsInner {
    fn take(&mut self) -> bool {
        if self.set {
            // The flag was set while we weren't looking.
            self.set = false;
            true
        } else {
            // `waiting` signals that the owner should be resumed when the flag is set. Copying
            // the task pointer is an optimization that allows clearing the
            // waiting state without computing the address of a separate field.
            self.waiting = Some(self.owner);

            false
        }
    }
}

/// A single event bit, optimized for the thread-mode embassy executor.
///
/// This takes shortcuts, which make it unsuitable for general purpose use (such as no wait
/// queue, no timeout, assumes a single thread waits for the flag, there is only a single bit of
/// flag information).
// 47
struct ThreadFlag {
    inner: NonReentrantMutex<FlagsInner>,
}

// 51
impl ThreadFlag {
    fn new() -> Self {
        let owner = SCHEDULER.with(|scheduler| {
            let current_cpu = Cpu::current() as usize;
            if let Some(current_task) = scheduler.per_cpu[current_cpu].current_task {
                current_task
            } else {
                // We're cheating, the task hasn't been initialized yet.
                NonNull::from(&scheduler.per_cpu[current_cpu].main_task)
            }
        });
        Self {
            inner: NonReentrantMutex::new(FlagsInner {
                owner,
                waiting: None,
                set: false,
            }),
        }
    }

    // 71
    fn with<R>(&self, f: impl FnOnce(&mut FlagsInner) -> R) -> R {
        self.inner.with(|inner| f(inner))
    }

    // 75
    fn set(&self) {
        self.with(|inner| {
            if let Some(waiting) = inner.waiting.take() {
                // The task is waiting, there is no need to set the flag - resuming the thread
                // is all the signal we need.
                /*
                unsafe {
                    info!("ThreadFlag::set - waiting.resume() {}", waiting.as_ref().name);
                }
                */
                waiting.resume();
            } else {
                // The task isn't waiting, set the flag.
                //info!("ThreadFlag::set - inner.set = true");
                inner.set = true;
            }
        });
    }

    // 88
    fn get(&self) -> bool {
        self.with(|inner| inner.set)
    }

    // 92
    fn wait(&self) {
        self.with(|inner| {
            if !inner.take() {
                // SCHEDULER.sleep_until, but we know the current task's ID, and we know there
                // is no timeout.
                SCHEDULER.with(|scheduler| {
                    /*
                    let forever_duration = Duration::from_secs(60 * 60 * 24 * 365 * 100); 
                    let forever_instant = Instant::now() + forever_duration;

                    info!("wait Instant::now() {:?}, forever_duration {:?}, sum {:?}",
                        Instant::now(), forever_duration, forever_instant);
                    */

                    /*
                    let ie = Instant::EPOCH;
                    info!("wait Instant::EPOCH {:?}, Duration::MAX {:?}, sum {:?}",
                        ie, Duration::MAX, Instant::EPOCH + Duration::MAX);
                    */

                    scheduler.sleep_task_until(inner.owner, Instant::EPOCH + Duration::MAX);
                    //scheduler.sleep_task_until(inner.owner, forever_instant);

                    /*
                    let now_u64 = Instant::now().duration_since_epoch().as_micros();
                    let dur_u64 = Duration::from_secs(3600).as_micros();
                    let sum_u64 = now_u64 + dur_u64;

                    info!("wait now_u64 {:?}, dur_u64 {:?}, sum {:?}",
                        now_u64, dur_u64, sum_u64);

                    scheduler.sleep_task_until(inner.owner, Instant::micros());
                    */

                    //info!("yield from wait after scheduler.sleep_task_until forever");
                    crate::task::yield_task();
                });
            }
        });
    }
}

// 106
#[unsafe(export_name = "__pender")]
#[ram]
fn __pender(context: *mut ()) {
    info!("__pender {:x}", context as usize);
    match context as usize {
        0 => unsafe { SoftwareInterrupt::<0>::steal().raise() },
        1 => unsafe { SoftwareInterrupt::<1>::steal().raise() },
        2 => unsafe { SoftwareInterrupt::<2>::steal().raise() },
        3 => unsafe { SoftwareInterrupt::<3>::steal().raise() },
        _ => {
            // This forces us to keep the embassy timer queue separate, otherwise we'd need to
            // reentrantly lock SCHEDULER.
            let flags = unwrap!(unsafe { context.cast::<ThreadFlag>().as_ref() });
            flags.set();
            //info!("after flags.set()");
        }
    }
}

/// Callbacks to run code before/after polling the task queue.
// 124
pub trait Callbacks {
    /// Called just before polling the executor.
    fn before_poll(&mut self);

    /// Called after the executor is polled, if there is no work scheduled.
    ///
    /// Note that tasks can become ready at any point during the execution
    /// of this function.
    fn on_idle(&mut self);
}

/// Thread-mode executor.
///
/// This executor runs in an OS thread, meaning the scheduler needs to be started before using any
/// async operations. If you wish to write async code without the scheduler running, consider
/// using the [`InterruptExecutor`].
#[cfg_attr(
    multi_core,
    doc = r"

If you want to start the executor on the second core, you will need to start the second core using [`crate::start_second_core`].
If you are looking for a way to run code on the second core without the scheduler, use the [`InterruptExecutor`].
"
)]
// 148
pub struct Executor {
    executor: UnsafeCell<MaybeUninit<raw::Executor>>,
}

// 152
impl Executor {
    /// Create a new thread-mode executor.
    pub const fn new() -> Self {
        Self {
            executor: UnsafeCell::new(MaybeUninit::uninit()),
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
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading
    ///   its lifetime with `transmute`. (unsafe)
    ///
    /// This function never returns.
    // 180
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        let flags = ThreadFlag::new();
        struct NoHooks;

        impl Callbacks for NoHooks {
            fn before_poll(&mut self) {}

            fn on_idle(&mut self) {}
        }

        self.run_inner(init, &flags, NoHooks)
    }

    /// Run the executor with callbacks.
    ///
    /// See [Callbacks] on when the callbacks are called.
    ///
    /// See [Self::run] for more information about running the executor.
    ///
    /// This function never returns.
    // 200
    pub fn run_with_callbacks(
        &'static mut self,
        init: impl FnOnce(Spawner),
        callbacks: impl Callbacks,
    ) -> ! {
        let flags = ThreadFlag::new();
        struct Hooks<'a, CB: Callbacks>(CB, &'a ThreadFlag);

        impl<CB: Callbacks> Callbacks for Hooks<'_, CB> {
            fn before_poll(&mut self) {
                self.0.before_poll()
            }

            fn on_idle(&mut self) {
                // Make sure we only call on_idle if the executor would otherwise go to sleep.
                if !self.1.get() {
                    self.0.on_idle();
                }
            }
        }

        self.run_inner(init, &flags, Hooks(callbacks, &flags))
    }

    // 224
    fn run_inner(
        &'static self,
        init: impl FnOnce(Spawner),
        flags: &ThreadFlag,
        mut hooks: impl Callbacks,
    ) -> ! {
        let executor = unsafe {
            (&mut *self.executor.get()).write(raw::Executor::new(
                (flags as *const ThreadFlag).cast::<()>().cast_mut(),
            ))
        };

        #[cfg(multi_core)]
        if Cpu::current() != Cpu::ProCpu
            && crate::SCHEDULER
                .with(|scheduler| !scheduler.per_cpu[Cpu::current() as usize].initialized)
        {
            panic!("Executor cannot be started: the scheduler is not running on the current CPU.");
        }

        init(executor.spawner());

        loop {
            hooks.before_poll();

            unsafe { executor.poll() };
            hooks.on_idle();

            // Wait for work to become available.
            flags.wait();
        }
    }
}

// 259
impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
