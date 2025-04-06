//182
#![no_std]

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

// 236
pub use procmacros::blocking_main as main;
