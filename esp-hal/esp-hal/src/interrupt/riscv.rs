pub use esp_riscv_rt::TrapFrame;
use riscv::register::mcause;

/// # Safety
///
/// This function is called from an assembly trap handler.
#[doc(hidden)]
#[link_section = ".trap.rust"]
#[export_name = "_start_trap_rust_hal"]
// 179
pub unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
    assert!(
        mcause::read().is_exception(),
        "Arrived into _start_trap_rust_hal but mcause is not an exception!"
    );
    extern "C" {
        fn ExceptionHandler(tf: *mut TrapFrame);
    }
    // either the DefaultExceptionHandler from esp-risv-rt or
    // the external ExceptionHandler function e.g. from esp-backtrace
    ExceptionHandler(trap_frame);
}

#[doc(hidden)]
#[no_mangle]
// 192
pub fn _setup_interrupts() {}
