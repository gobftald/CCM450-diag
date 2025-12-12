#[cfg(feature = "backtrace")]
use esp_backtrace as _;

#[cfg(not(feature = "backtrace"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    //use core::fmt::Write;
    //core_println!("{}", info);
    esp_println::println!("{}", info);

    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}
