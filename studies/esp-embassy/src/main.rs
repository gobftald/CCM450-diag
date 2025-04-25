#![no_std]
#![no_main]
// for 'task' embassy-executor-macro, when embassy-executor/nightly
#![feature(impl_trait_in_assoc_type)]
// for using #[thread_local] in 'task' embassy-executor-macro
// to make 'static POOL: ... = embassy_executor::raw::TaskPool::new() 'unsync'
#![feature(thread_local)]

// panic_handler
mod panic;

#[allow(unused_imports)]
#[macro_use(core_println, println, debug, panic)] // core_println for panic_handler in mod panic
extern crate console;

/*
#[embassy_executor::task]
async fn run() {
    loop {
        /*
        esp_println::println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
        */
    }
}
*/

async fn __run_task() {
    loop {}
}

fn run() -> ::embassy_executor::SpawnToken {
    trait _EmbassyInternalTaskTrait {
        type Fut: ::core::future::Future + 'static;
        fn construct() -> Self::Fut;
    }
    impl _EmbassyInternalTaskTrait for () {
        type Fut = impl core::future::Future + 'static;
        fn construct() -> Self::Fut {
            __run_task()
        }
    }
    const POOL_SIZE: usize = 1;
    //#[thread_local]
    static mut POOL: ::embassy_executor::raw::TaskPool<
        <() as _EmbassyInternalTaskTrait>::Fut,
        POOL_SIZE,
    > = ::embassy_executor::raw::TaskPool::new();
    unsafe { POOL._spawn_async_fn(move || <() as _EmbassyInternalTaskTrait>::construct()) }
}

#[esp_hal::main]
fn main() -> ! {
    console::print(b"01234567890123456789012345678901234567890123456789012345678\n");

    println!("println from main 0x{:x}", 0x55);
    debug!("debug {=u32} 0x{:x}", 0x66, 0x77);

    // for testing exception
    unsafe {
        core::arch::asm!("unimp");
    }

    panic!("panic called from main")
}

// #build-std-features = ["panic_immediate_abort"], defmt off, console on/off
// cargo run --release
// cargo run --release --features=backtrace     # beacktrace switch console on by default
//                                              # it needs the console in all cases

// both 'build-std-features = ["panic_immediate_abort"]' and defmt are on
// cargo run --release
// cargo run --release --features=backtrace     # "panic_immediate_abort" off, "force-frame-pointers" on
//                                              # defmt on/off
