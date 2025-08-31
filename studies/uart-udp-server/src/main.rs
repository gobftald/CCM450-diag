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

mod uart;
mod udp;

pub const CHANNEL_ITEM_SIZE: usize = 64;
pub const CHANNEL_ITEMS_MAX: usize = 1;

use core::cell::OnceCell;
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static mut STATIC_CELL: OnceCell<$t> = OnceCell::new();
        unsafe { STATIC_CELL.get_mut_or_init(|| $val) }
    }};
}

#[derive(Clone, Copy)]
pub struct ChannelItem {
    size: u8,
    data: [u8; CHANNEL_ITEM_SIZE - size_of::<u8>()],
}

impl ChannelItem {
    const fn empty() -> Self {
        Self {
            size: 0,
            data: [0; CHANNEL_ITEM_SIZE - size_of::<u8>()],
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    // WIFI setup
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = &*mk_static!(
        esp_wifi::EspWifiController<'static>,
        unwrap!(esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK))
    );

    let (controller, interfaces) = unwrap!(esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI));

    let wifi_ap_device = interfaces.ap;

    use core::net::Ipv4Addr;
    let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Addr::new(192, 168, 3, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 3, 1)),
        dns_servers: Default::default(),
    });

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    use embassy_net::StackResources;
    let (ap_stack, ap_runner) = embassy_net::new(
        wifi_ap_device,
        ap_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );
    // EOF WIFI setup

    // Channel Setup for communication between UDP and UART
    use embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel};
    // we can use NoopRawMutex since we use channel between two tasks in the same executor,
    // in single core environment and not using from interrupt
    //use esp_hal::sync::RawMutex;

    let udp2uart_buffer = mk_static!(
        [ChannelItem; CHANNEL_ITEMS_MAX],
        [ChannelItem::empty(); CHANNEL_ITEMS_MAX]
    );
    let udp2uart_channel = mk_static!(
        Channel<'_, NoopRawMutex, ChannelItem>,
        Channel::new(udp2uart_buffer)
    );
    let (udp_sender, uart_receiver) = udp2uart_channel.split();

    let uart2udp_buffer = mk_static!(
        [ChannelItem; CHANNEL_ITEMS_MAX],
        [ChannelItem::empty(); CHANNEL_ITEMS_MAX]
    );
    let uart2udp_channel = mk_static!(
        Channel<'_, NoopRawMutex, ChannelItem>,
        Channel::new(uart2udp_buffer)
    );
    let (uart_sender, udp_receiver) = uart2udp_channel.split();
    // EOF Channel setup

    spawner
        .spawn(udp::server(controller, ap_stack, udp_sender, udp_receiver))
        .ok();
    spawner
        .spawn(uart::client(
            uart_sender,
            uart_receiver,
            peripherals.UART0,
            peripherals.GPIO20,
            peripherals.GPIO21,
        ))
        .ok();
    spawner.spawn(net_task(ap_runner)).ok();
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

#[embassy_executor::task()]
pub async fn net_task(
    mut runner: embassy_net::Runner<'static, esp_wifi::wifi::WifiDevice<'static>>,
) {
    runner.run().await
}
