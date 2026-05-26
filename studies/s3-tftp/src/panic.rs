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

    defmt::error!("PANIC OCCURRED: {}", defmt::Display2Format(info));

    loop {
        #[cfg(target_arch = "riscv32")]
        unsafe { core::arch::asm!("wfi") }

        #[cfg(target_arch = "xtensa")]
        unsafe { core::arch::asm!("waiti 0") }
    }
}

/* AI chat resetting spi after panic
let's get back to the unsuccessful init sequence, which was not able to kick off the middle of state machine after panic
15:49
Claude responded: Right.
Right. The dummy clock approach failed because the card wasn't just uninitialized — it was mid-transaction with CS potentially asserted when the panic hit. Dummy clocks with CS high only reset the card if it's in idle state, not mid-command.
The real fix is to ensure clean shutdown on panic. You need a panic handler that:
Forces CS high immediately
Sends enough clocks to flush any pending transaction
Then resets
The problem with doing this in the panic handler is you don't have safe access to the SPI peripheral or CS pin — they're owned by your task.
The cleanest solution is to use a static atomic flag for the CS pin number and manipulate it via direct register writes in the panic handler:
rust
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // GPIO41 = SD CS, force high via W1TS register
        // GPIO bank 1 (GPIOs 32-63)
        const GPIO_OUT1_W1TS: *mut u32 = 0x6000_4010 as *mut u32;
        core::ptr::write_volatile(GPIO_OUT1_W1TS, 1 << (41 - 32));

        // bit-bang 80 dummy clocks on SCK (GPIO39) with MOSI high
        const GPIO_OUT1_W1TS_REG: *mut u32 = 0x6000_4010 as *mut u32;
        const GPIO_OUT1_W1TC_REG: *mut u32 = 0x6000_4014 as *mut u32;
        let sck_bit = 1u32 << (39 - 32);

        for _ in 0..80 {
            core::ptr::write_volatile(GPIO_OUT1_W1TC_REG, sck_bit); // SCK low
            for _ in 0..10 { core::hint::spin_loop(); } // small delay
            core::ptr::write_volatile(GPIO_OUT1_W1TS_REG, sck_bit); // SCK high
            for _ in 0..10 { core::hint::spin_loop(); }
        }
    }

    defmt::error!("panic: {}", defmt::Display2Format(info));
    esp_hal::reset::software_reset();
}
Do you already have a custom panic handler, or are you using the default one from esp-hal?
*/