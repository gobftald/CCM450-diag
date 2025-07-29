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
use embassy_net::StackResources;
use embassy_time::{Duration, Timer};

use core::cell::OnceCell;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

// 89
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static mut STATIC_CELL: OnceCell<$t> = OnceCell::new();
        unsafe { STATIC_CELL.get_mut_or_init(|| $val) }
    }};
}

//const GW_IP_ADDR_ENV: Option<&'static str> = option_env!("GATEWAY_IP");

#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
    //let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = &*mk_static!(
        esp_wifi::EspWifiController<'static>,
        unwrap!(esp_wifi::init(
            timg0.timer0,
            rng.clone(),
            peripherals.RADIO_CLK
        ))
    );

    let (mut controller, interfaces) =
        unwrap!(esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI));

    let wifi_ap_device = interfaces.ap;
    let wifi_sta_device = interfaces.sta;

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Addr::new(192, 168, 3, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 3, 1)),
        dns_servers: Default::default(),
    });
    let sta_config = embassy_net::Config::dhcpv4(Default::default());

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stacks
    let (ap_stack, ap_runner) = embassy_net::new(
        wifi_ap_device,
        ap_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );
    let (sta_stack, sta_runner) = embassy_net::new(
        wifi_sta_device,
        sta_config,
        mk_static!(StackResources<4>, StackResources::<4>::new()),
        seed,
    );

    let client_config = esp_wifi::wifi::Configuration::Mixed(
        esp_wifi::wifi::ClientConfiguration {
            ssid: SSID.into(),
            password: PASSWORD.into(),
            auth_method: esp_wifi::wifi::AuthMethod::WPA2Personal,
            ..Default::default()
        },
        esp_wifi::wifi::AccessPointConfiguration {
            ssid: "CCM-GP450".into(),
            ..Default::default()
        },
    );
    unwrap!(controller.set_configuration(&client_config));

    /*
    unsafe {
        debug!("{}", esp_alloc::HEAP.stats());
    }
    */

    spawner.spawn(connection(controller)).ok();
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

#[embassy_executor::task]
async fn connection(mut controller: esp_wifi::wifi::WifiController<'static>) {
    debug!("start connection task");
    debug!("Device capabilities: {:?}", controller.capabilities());

    debug!("Starting wifi");
    controller.start_async().await.unwrap();
    debug!("Wifi started!");

    loop {
        match esp_wifi::wifi::ap_state() {
            esp_wifi::wifi::WifiState::ApStarted => {
                debug!("About to connect...");

                match controller.connect_async().await {
                    Ok(_) => {
                        debug!("WifiEvent::StaConnected");

                        // wait until we're no longer connected
                        controller
                            .wait_for_event(esp_wifi::wifi::WifiEvent::StaDisconnected)
                            .await;
                        debug!("STA disconnected");
                    }
                    Err(e) => {
                        debug!("Failed to connect to wifi: {:?}", e);
                        Timer::after(Duration::from_millis(5000)).await
                    }
                }
            }
            _ => {
                debug!("SoftAP failed to start");
                return;
            }
        }
    }
}

// cmd line
// SSID="iWIFI-2.4" PASSWORD="dingomolarsanta" DEFMT_LOG=debug \
// cargo run --release --features=backtrace,esp-wifi/sys-logs
