#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod panic_handlers;

#[embassy_executor::task]
async fn run() {}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {}
