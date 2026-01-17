// 678
#![no_std]

// 681
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
mod asm;

// 696
#[cfg(not(feature = "s-mode"))]
use riscv::register::mcause as xcause;

// 702
pub use riscv_rt_macros::{core_interrupt, entry};
pub use riscv_types::*;

// 707
#[cfg(feature = "pre-init")]
pub use riscv_rt_macros::pre_init;

/// We export this static with an informative name so that if an application attempts to link
/// two copies of riscv-rt together, linking will fail. We also declare a links key in
/// Cargo.toml which is the more modern way to solve the same problem, but we have to keep
/// __ONCE__ around to prevent linking with versions before the links key was added.
// 714
#[export_name = "error: riscv-rt appears more than once in the dependency graph"]
#[doc(hidden)]
pub static __ONCE__: () = ();

/// Rust entry point (_start_rust)
///
/// Configures interrupts and calls main. This function never returns.
///
/// # Safety
///
/// This function should not be called directly by the user, and should instead
/// be invoked by the runtime implicitly.
// 851
#[cfg_attr(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    link_section = ".init.rust"
)]
#[export_name = "_start_rust"]
pub unsafe extern "C" fn start_rust(a0: usize, a1: usize, a2: usize) -> ! {
    extern "Rust" {
        #[cfg(feature = "post-init")]
        fn __post_init(a0: usize);
        fn _setup_interrupts();
        fn hal_main(a0: usize, a1: usize, a2: usize) -> !;
    }

    #[cfg(feature = "post-init")]
    __post_init(a0);
    _setup_interrupts();
    hal_main(a0, a1, a2);
}

/// Registers saved in trap handler
// 777
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TrapFrame {
    /// `x1`: return address, stores the address to return to after a function call or interrupt.
    pub ra: usize,
    /// `x5`: temporary register `t0`, used for intermediate values.
    pub t0: usize,
    /// `x6`: temporary register `t1`, used for intermediate values.
    pub t1: usize,
    /// `x7`: temporary register `t2`, used for intermediate values.
    pub t2: usize,
    /// `x28`: temporary register `t3`, used for intermediate values.
    #[cfg(riscvi)]
    pub t3: usize,
    /// `x29`: temporary register `t4`, used for intermediate values.
    #[cfg(riscvi)]
    pub t4: usize,
    /// `x30`: temporary register `t5`, used for intermediate values.
    #[cfg(riscvi)]
    pub t5: usize,
    /// `x31`: temporary register `t6`, used for intermediate values.
    #[cfg(riscvi)]
    pub t6: usize,
    /// `x10`: argument register `a0`. Used to pass the first argument to a function.
    pub a0: usize,
    /// `x11`: argument register `a1`. Used to pass the second argument to a function.
    pub a1: usize,
    /// `x12`: argument register `a2`. Used to pass the third argument to a function.
    pub a2: usize,
    /// `x13`: argument register `a3`. Used to pass the fourth argument to a function.
    pub a3: usize,
    /// `x14`: argument register `a4`. Used to pass the fifth argument to a function.
    pub a4: usize,
    /// `x15`: argument register `a5`. Used to pass the sixth argument to a function.
    pub a5: usize,
    #[cfg(riscvi)]
    /// `x16`: argument register `a6`. Used to pass the seventh argument to a function.
    pub a6: usize,
    #[cfg(riscvi)]
    /// `x17`: argument register `a7`. Used to pass the eighth argument to a function.
    pub a7: usize,
}

/// Trap entry point rust (_start_trap_rust)
///
/// `scause`/`mcause` is read to determine the cause of the trap. XLEN-1 bit indicates
/// if it's an interrupt or an exception. The result is examined and one of the
/// exception handlers or one of the core interrupt handlers is called.
///
/// # Note
///
/// Exception dispatching is performed by an extern `_dispatch_exception` function.
/// Targets that comply with the RISC-V standard can use the implementation provided
/// by this crate in the [`exceptions`] module. Targets with special exception sources
/// may provide their custom implementation of the `_dispatch_exception` function. You may
/// also need to enable the `no-exceptions` feature to op-out the default implementation.
///
/// In direct mode (i.e., `v-trap` feature disabled), interrupt dispatching is performed
/// by an extern `_dispatch_core_interrupt` function. Targets that comply with the RISC-V
/// standard can use the implementation provided by this crate in the [`interrupts`] module.
/// Targets with special interrupt sources may provide their custom implementation of the
/// `_dispatch_core_interrupt` function. You may also need to enable the `no-interrupts`
/// feature to op-out the default implementation.
///
/// In vectored mode (i.e., `v-trap` feature enabled), interrupt dispatching is performed
/// directly by hardware, and thus this function should **not** be triggered due to an
/// interrupt. If this abnormal situation happens, this function will directly call the
/// `DefaultHandler` function.
///
/// # Safety
///
/// This function must be called only from assembly `_start_trap` function.
/// Do **NOT** call this function directly.
// 8
#[cfg_attr(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    link_section = ".trap.rust"
)]
#[export_name = "_start_trap_rust"]
pub unsafe extern "C" fn start_trap_rust(trap_frame: *const TrapFrame) {
    extern "C" {
        #[cfg(not(feature = "v-trap"))]
        fn _dispatch_core_interrupt(code: usize);
        #[cfg(feature = "v-trap")]
        fn DefaultHandler();
        fn _dispatch_exception(trap_frame: &TrapFrame, code: usize);
    }

    match xcause::read().cause() {
        #[cfg(not(feature = "v-trap"))]
        xcause::Trap::Interrupt(code) => _dispatch_core_interrupt(code),
        #[cfg(feature = "v-trap")]
        xcause::Trap::Interrupt(_) => DefaultHandler(),
        xcause::Trap::Exception(code) => _dispatch_exception(&*trap_frame, code),
    }
}
