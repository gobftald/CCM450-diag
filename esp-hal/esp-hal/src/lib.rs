#![no_std]

#[cfg(riscv)]
pub use esp_riscv_rt::entry;
