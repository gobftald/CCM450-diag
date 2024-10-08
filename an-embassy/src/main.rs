#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;

mod panic_handlers;

/*
#[embassy_executor::task]
async fn run() {}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {}
*/

async fn __run_task() {}

fn run() -> ::embassy_executor::SpawnToken<impl Sized> {
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
    static POOL: ::embassy_executor::raw::TaskPool<
        <() as _EmbassyInternalTaskTrait>::Fut,
        POOL_SIZE,
    > = ::embassy_executor::raw::TaskPool::new();
    unsafe { POOL._spawn_async_fn(move || <() as _EmbassyInternalTaskTrait>::construct()) }
}

async fn ____embassy_main_task(spawner: Spawner) {
    {}
}

fn __embassy_main(spawner: Spawner) -> ::embassy_executor::SpawnToken<impl Sized> {
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
    static POOL: ::embassy_executor::raw::TaskPool<
        <() as _EmbassyInternalTaskTrait>::Fut,
        POOL_SIZE,
    > = ::embassy_executor::raw::TaskPool::new();
    unsafe { POOL._spawn_async_fn(move || <() as _EmbassyInternalTaskTrait>::construct(spawner)) }
}

unsafe fn __make_static<T>(t: &mut T) -> &'static mut T {
    ::core::mem::transmute(t)
}

#[allow(non_snake_case)]
#[export_name = "main"]
pub fn __risc_v_rt__main() -> ! {
    let mut executor = ::esp_hal_embassy::Executor::new();
    let executor = unsafe { __make_static(&mut executor) };
    executor.run(|spawner| {
        spawner.must_spawn(__embassy_main(spawner));
    })
}
