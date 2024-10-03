pub use self::thread::*;

mod thread;

#[export_name = "__pender"]
fn __pender(context: *mut ()) {
    let context = (context as usize).to_le_bytes();
    // context[1] = get_core()
    thread::pend_thread_mode(context[1] as usize)
}
