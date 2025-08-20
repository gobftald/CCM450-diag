#![no_std]
#![no_main]
// for 'task' embassy-executor-macro, when embassy-executor/nightly
#![feature(impl_trait_in_assoc_type)]
#![feature(once_cell_get_mut)]
#![allow(static_mut_refs)]

// panic_handler
mod panic;

#[macro_use(core_println, unwrap, debug)] // core_println for panic_handler in mod panic
extern crate console;

use core::cell::OnceCell;
use core::net::Ipv4Addr;

use embassy_net::StackResources;

mod udp;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static mut STATIC_CELL: OnceCell<$t> = OnceCell::new();
        unsafe { STATIC_CELL.get_mut_or_init(|| $val) }
    }};
}

#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = &*mk_static!(
        esp_wifi::EspWifiController<'static>,
        unwrap!(esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK))
    );

    let (controller, interfaces) = unwrap!(esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI));

    let wifi_ap_device = interfaces.ap;

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Addr::new(192, 168, 3, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 3, 1)),
        dns_servers: Default::default(),
    });

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (ap_stack, ap_runner) = embassy_net::new(
        wifi_ap_device,
        ap_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(udp::server(controller, ap_stack)).ok();
    spawner.spawn(udp::net_task(ap_runner)).ok();
    spawner.spawn(run()).ok();
}

#[embassy_executor::task]
async fn run() {
    loop {
        //info!("Hello world from embassy using esp-hal-async!");
        use core::fmt::Write;
        core_println!("0");
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1_000)).await;
    }
}
