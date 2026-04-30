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
fn core_panic(_: &core::panic::PanicInfo) -> ! {
    // defmt::panic!(...) prints the log messages then calls defmt::export::panic()
    // defmt::export::panic() calls _defmt_panic()
    //
    // there is no PanicInfo argument anywhere, so we don't get info about where was the original
    // panic called (hence no any core formatting triggered)
    //
    // when there is no defmt::panic_handler -> PROVIDE(_defmt_panic = __defmt_default_panic);
    //
    // __defmt_default_panic is the export name of  'fn default_panic()' which calls core::panic!()
    // so we would get PanicInfo from the place where core::panic! was called in the defmt crate
    //
    // that's why we don't show this meaningless information

    loop {
        #[cfg(target_arch = "riscv32")]
        unsafe { core::arch::asm!("wfi") }

        #[cfg(target_arch = "xtensa")]
        unsafe { core::arch::asm!("waiti 0") }
    }
}
