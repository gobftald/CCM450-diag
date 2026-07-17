#[cfg(feature = "backtrace")]
use esp_backtrace as _;

#[cfg(all(not(feature = "backtrace"), not(feature = "defmt")))]
#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("{}", info);

    loop {
        #[cfg(target_arch = "riscv32")]
        unsafe { core::arch::asm!("wfi") }

        #[cfg(target_arch = "xtensa")]
        unsafe { core::arch::asm!("waiti 0") }
    }
}

#[cfg(all(not(feature = "backtrace"), feature = "defmt"))]
#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("PANIC OCCURRED: {}", defmt::Display2Format(info));

    loop {
        #[cfg(target_arch = "riscv32")]
        unsafe { core::arch::asm!("wfi") }

        #[cfg(target_arch = "xtensa")]
        unsafe { core::arch::asm!("waiti 0") }
    }
}
