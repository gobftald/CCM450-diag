#![no_std]
#![no_main]
// for 'task' embassy-executor-macro, when embassy-executor/nightly
#![feature(impl_trait_in_assoc_type)]
#![allow(static_mut_refs)]

// panic_handler
mod panic;

#[allow(unused_imports)]
#[macro_use(core_println, info)] // core_println for panic_handler in mod panic
extern crate console;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
async fn run() {
    loop {
        //info!("Hello world from embassy using esp-hal-async!");
        use core::fmt::Write;
        core_println!("0");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    //let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    spawner.spawn(run()).ok();
}

/*
async fn __run_task() {
    {
        loop {
            info!("Hello world from embassy using esp-hal-async!");
            use core::fmt::Write;
            core_println!("0");
            Timer::after(Duration::from_millis(1_000)).await;
        }
    }
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

    //static POOL: ::embassy_executor::raw::TaskPool<
    static mut POOL: ::embassy_executor::raw::TaskPool<
        <() as _EmbassyInternalTaskTrait>::Fut,
        POOL_SIZE,
    > = ::embassy_executor::raw::TaskPool::new();

    //unsafe { POOL._spawn_async_fn(move || <() as _EmbassyInternalTaskTrait>::construct()) }
    unsafe { POOL.spawn(move || <() as _EmbassyInternalTaskTrait>::construct()) }
}

async fn ____embassy_main_task(spawner: Spawner) {
    {
        //let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
        let peripherals = esp_hal::init(config);

        let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
        esp_hal_embassy::init(systimer.alarm0);

        spawner.spawn(run()).ok();
    }
}

fn __embassy_main(spawner: Spawner) -> ::embassy_executor::SpawnToken {
    trait _EmbassyInternalTaskTrait {
        type Fut: ::core::future::Future + 'static;
        fn construct(spawner: Spawner) -> Self::Fut;
    }

    impl _EmbassyInternalTaskTrait for () {
        type Fut = impl core::future::Future + 'static;
        fn construct(spawner: Spawner) -> Self::Fut {
            ____embassy_main_task(spawner)
        }
    }

    const POOL_SIZE: usize = 1;

    //static POOL: ::embassy_executor::raw::TaskPool<
    static mut POOL: ::embassy_executor::raw::TaskPool<
        <() as _EmbassyInternalTaskTrait>::Fut,
        POOL_SIZE,
    > = ::embassy_executor::raw::TaskPool::new();

    //unsafe { POOL._spawn_async_fn(move || <() as _EmbassyInternalTaskTrait>::construct(spawner)) }
    unsafe { POOL.spawn(move || <() as _EmbassyInternalTaskTrait>::construct(spawner)) }
}

#[doc(hidden)]
unsafe fn __make_static<T>(t: &mut T) -> &'static mut T {
    ::core::mem::transmute(t)
}

#[allow(non_snake_case)]
#[esp_hal::main]
fn __risc_v_rt__main() -> ! {
    let mut executor = ::esp_hal_embassy::Executor::new();
    let executor = unsafe { __make_static(&mut executor) };

    executor.run(|spawner| {
        spawner.must_spawn(__embassy_main(spawner));
    })
}
*/
