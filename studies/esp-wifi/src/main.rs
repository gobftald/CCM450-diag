#![no_std]
#![no_main]
// for 'task' embassy-executor-macro, when embassy-executor/nightly
#![feature(impl_trait_in_assoc_type)]
// for 'single thread' statics
#![allow(static_mut_refs)]
#![feature(once_cell_get_mut)]

// panic_handler
mod panic;

#[allow(unused_imports)]
#[macro_use(core_println, debug, unwrap)] // core_println for panic_handler in mod panic
extern crate console;

use core::net::Ipv4Addr;

use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, StaticConfigV4};
use embassy_time::{Duration, Timer};

use core::cell::OnceCell;
// 89
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static mut STATIC_CELL: OnceCell<$t> = OnceCell::new();
        unsafe { STATIC_CELL.get_mut_or_init(|| $val) }
    }};
}

//const GW_IP_ADDR_ENV: Option<&'static str> = option_env!("GATEWAY_IP");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    //let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = &*mk_static!(
        esp_wifi::EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );

    let (mut controller, interfaces) =
        esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();

    /*
    unsafe {
        debug!("{}", esp_alloc::HEAP.stats());
    }
    */

    let wifi_ap_device = interfaces.ap;
    let wifi_sta_device = interfaces.sta;

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let ap_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Addr::new(192, 168, 3, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 3, 1)),
        dns_servers: Default::default(),
    });
    let sta_config = embassy_net::Config::dhcpv4(Default::default());

    spawner.spawn(run()).ok();
}

#[embassy_executor::task]
async fn run() {
    loop {
        //info!("Hello world from embassy using esp-hal-async!");
        use core::fmt::Write;
        core_println!("0");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}
