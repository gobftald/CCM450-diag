pub use esp_riscv_rt::TrapFrame;
use riscv::register::{mcause, mtvec};

/// # Safety
///
/// This function is called from an assembly trap handler.
#[link_section = ".trap.rust"]
#[export_name = "_start_trap_rust_hal"]
// 216
pub unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
    // if assert failed and if build-std-features = ["panic_immediate_abort"]
    // this is a forever loop
    // assert makes panic, but panic at "panic_immediate_abort" concludes unimp
    // exception, which comes back here again
    //
    // in defmt case this makes the error message repeated forever
    // in no defmt case the forever loop is running, but no error messages
    //
    // fortunately this is only a rare  (theoretical) error case
    assert!(
        mcause::read().is_exception(),
        "Arrived into _start_trap_rust_hal but mcause is not an exception!"
    );

    extern "C" {
        fn ExceptionHandler(tf: *mut TrapFrame);
    }
    ExceptionHandler(trap_frame);
}

#[no_mangle]
// 229
pub fn _setup_interrupts() {
    extern "C" {
        static _vector_table: *const u32;
    }

    unsafe {
        // disable all known interrupts
        // at least after the 2nd stage bootloader there are some interrupts enabled
        // (e.g. UART)

        let vec_table = &_vector_table as *const _ as usize;
        mtvec::write(vec_table, mtvec::TrapMode::Vectored);
    }
}
