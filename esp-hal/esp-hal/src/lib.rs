//182
#![no_std]

// 236
pub use procmacros::blocking_main as main;

#[cfg(any(/*dport,*/ interrupt_core0, /*interrupt_core1*/))]
// 293
pub mod interrupt;

#[doc(hidden)]
// 486
pub mod __macro_implementation {
    #[cfg(riscv)]
    // 496
    pub use esp_riscv_rt::entry as __entry;
    #[cfg(xtensa)]
    // 498
    pub use xtensa_lx_rt::entry as __entry;
}

#[cfg(riscv)]
#[export_name = "hal_main"]
// 503
fn hal_main(a0: usize, a1: usize, a2: usize) -> ! {
    extern "Rust" {
        // This symbol will be provided by the user via `#[entry]`
        fn main(a0: usize, a1: usize, a2: usize) -> !;
    }

    unsafe {
        main(a0, a1, a2);
    }
}
