#[cfg_attr(target_arch = "riscv32", path = "preempt_riscv.rs")]
mod arch_specific;
pub mod timer;

// 6
use core::{ffi::c_void, mem::MaybeUninit};

// 8
use allocator_api2::boxed::Box;
use arch_specific::*;
pub(crate) use timer::setup_timer;

// 11
use timer::setup_multitasking;

// 13
use crate::{hal::trapframe::TrapFrame, preempt::Scheduler};

// 20
struct Context {
    trap_frame: TrapFrame,
    pub thread_semaphore: u32,
    pub next: *mut Context,
    //pub _allocated_stack: Box<[MaybeUninit<u8>], InternalMemory>,
    pub _allocated_stack: Box<[MaybeUninit<u8>]>,
}

// 27
impl Context {
    pub(crate) fn new(
        task_fn: extern "C" fn(*mut c_void),
        param: *mut c_void,
        task_stack_size: usize,
    ) -> Self {
        trace!("task_create {:?} {:?} {}", task_fn, param, task_stack_size);

        //let mut stack = Box::<[u8], _>::new_uninit_slice_in(task_stack_size, InternalMemory);
        let mut stack = Box::<[u8], _>::new_uninit_slice(task_stack_size);

        let stack_top = unsafe { stack.as_mut_ptr().add(task_stack_size).cast() };

        Context {
            trap_frame: new_task_context(task_fn, param, stack_top),
            thread_semaphore: 0,
            next: core::ptr::null_mut(),
            _allocated_stack: stack,
        }
    }
}

// 48
struct SchedulerState {
    /// Pointer to the current task.
    ///
    /// Tasks are stored in a circular linked list. CTX_NOW points to the
    /// current task.
    current_task: *mut Context,

    /// Pointer to the task that is scheduled for deletion.
    to_delete: *mut Context,
}

// 59
impl SchedulerState {
    // 60
    const fn new() -> Self {
        Self {
            current_task: core::ptr::null_mut(),
            to_delete: core::ptr::null_mut(),
        }
    }

    // 67
    fn delete_task(&mut self, task: *mut Context) {
        let mut current_task = self.current_task;
        // Save the first pointer so we can prevent an accidental infinite loop.
        let initial = current_task;
        loop {
            // We don't have the previous pointer, so we need to walk forward in the circle
            // even if we need to delete the first task.

            // If the next task is the one we want to delete, we need to remove it from the
            // list, then drop it.
            let next_task = unsafe { (*current_task).next };
            if core::ptr::eq(next_task, task) {
                unsafe {
                    (*current_task).next = (*next_task).next;

                    core::ptr::drop_in_place(task);
                    break;
                }
            }

            // If the next task is the first task, we can stop. If we needed to delete the
            // first task, we have already handled it in the above case. If we needed to
            // delete another task, it has already been deleted in a previous iteration.
            if core::ptr::eq(next_task, initial) {
                break;
            }

            // Move to the next task.
            current_task = next_task;
        }
    }

    // 99
    fn switch_task(&mut self, trap_frame: &mut TrapFrame) {
        save_task_context(unsafe { &mut *self.current_task }, trap_frame);

        if !self.to_delete.is_null() {
            let task_to_delete = core::mem::replace(&mut self.to_delete, core::ptr::null_mut());
            self.delete_task(task_to_delete);
        }

        unsafe { self.current_task = (*self.current_task).next };

        restore_task_context(unsafe { &mut *self.current_task }, trap_frame);
    }
}

// 118
//static SCHEDULER_STATE: Locked<SchedulerState> = Locked::new(SchedulerState::new());
// The original Locked type his is largely equivalent to a `Mutex<RefCell<T>>`,
// but accessing the inner data doesn't hold a critical section on multi-core systems.
// So we will totally skip its 'with' function
static mut SCHEDULER_STATE: SchedulerState = SchedulerState::new();

// 120
struct BuiltinScheduler {}

// 122
crate::scheduler_impl!(static SCHEDULER: BuiltinScheduler = BuiltinScheduler {});

// 124
impl Scheduler for BuiltinScheduler {
    // 125
    fn enable(&self) {
        // allocate the main task
        allocate_main_task();
        setup_multitasking();
    }

    // 137
    fn yield_task(&self) {
        timer::yield_task()
    }

    // 141
    fn task_create(
        &self,
        task: extern "C" fn(*mut c_void),
        param: *mut c_void,
        task_stack_size: usize,
    ) -> *mut c_void {
        //let task = Box::new_in(Context::new(task, param, task_stack_size), InternalMemory);
        let task = Box::new(Context::new(task, param, task_stack_size));
        let task_ptr = Box::into_raw(task);

        //SCHEDULER_STATE.with(|state| unsafe {
        //let current_task = state.current_task;
        let current_task = unsafe { SCHEDULER_STATE.current_task };

        debug_assert!(
            !current_task.is_null(),
            "Tried to allocate a task before allocating the main task"
        );

        // Insert the new task at the next position.
        unsafe {
            let next = (*current_task).next;
            (*task_ptr).next = next;
            (*current_task).next = task_ptr;
        }
        //});

        task_ptr as *mut c_void
    }

    // 165
    fn current_task(&self) -> *mut c_void {
        current_task() as *mut c_void
    }

    // 173
    fn current_task_thread_semaphore(&self) -> *mut crate::binary::c_types::c_void {
        unsafe {
            &mut ((*current_task()).thread_semaphore) as *mut _
                as *mut crate::binary::c_types::c_void
        }
    }
}

// 181
fn allocate_main_task() {
    // This context will be filled out by the first context switch.
    //let context = Box::new_in(
    let context = Box::new(
        Context {
            trap_frame: TrapFrame::default(),
            thread_semaphore: 0,
            next: core::ptr::null_mut(),
            //_allocated_stack: Box::<[u8], _>::new_uninit_slice_in(0, InternalMemory),
            _allocated_stack: Box::<[u8]>::new_uninit_slice(0),
        },
        //InternalMemory,
    );

    let context_ptr = Box::into_raw(context);
    unsafe {
        // The first task loops back to itself.
        (*context_ptr).next = context_ptr;
    }

    //SCHEDULER_STATE.with(|state| {
    debug_assert!(
        unsafe { SCHEDULER_STATE.current_task.is_null() },
        "Tried to allocate main task multiple times",
    );
    //});

    unsafe {
        SCHEDULER_STATE.current_task = context_ptr;
    }
    //})
}

// 240
fn current_task() -> *mut Context {
    //SCHEDULER_STATE.with(|state| state.current_task)
    unsafe { SCHEDULER_STATE.current_task }
}

// 256
pub(crate) fn task_switch(trap_frame: &mut TrapFrame) {
    //SCHEDULER_STATE.with(|state| state.switch_task(trap_frame));
    unsafe {
        SCHEDULER_STATE.switch_task(trap_frame);
    }
}
