#![no_std]
#![no_main]
// for 'task' embassy-executor-macro, when embassy-executor/nightly
#![feature(impl_trait_in_assoc_type)]
#![feature(once_cell_get_mut)]
#![allow(static_mut_refs)]
#![feature(ascii_char)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

// panic_handler
mod panic;

mod adapter;
mod debug_pin;
mod ecu;
mod udp;

#[cfg(target_arch = "riscv32")]
use esp_hal::interrupt::software::SoftwareInterruptControl;

// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel};

const CHANNEL_ITEM_SIZE: usize = 64 - size_of::<usize>();
const CHANNEL_ITEMS_MAX: usize = 1;

#[derive(Clone, Copy)]
pub struct ChannelItem {
    pub size: usize,
    pub data: [u8; CHANNEL_ITEM_SIZE],
}

impl ChannelItem {
    const fn empty() -> Self {
        Self {
            size: 0,
            data: [0; CHANNEL_ITEM_SIZE],
        }
    }
}

use core::cell::OnceCell;
#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static mut STATIC_CELL: OnceCell<$t> = OnceCell::new();
        unsafe { STATIC_CELL.get_mut_or_init(|| $val) }
    }};
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config = esp_hal::Config::new_and_default(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    //esp_alloc::heap_allocator!(size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 96 * 1024);

    //let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
    //esp_hal_embassy::init(systimer.alarm0);

    // WIFI setup
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    #[cfg(target_arch = "riscv32")]
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        sw_int.software_interrupt0,
    );

    let esp_radio_ctrl = &*mk_static!(esp_radio::Controller<'static>, unwrap!(esp_radio::init()));

    let (controller, interfaces) = unwrap!(esp_radio::wifi::new(esp_radio_ctrl, peripherals.WIFI));

    let wifi_ap_device = interfaces.ap;

    use core::net::Ipv4Addr;
    let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Addr::new(192, 168, 3, 1), 24),
        gateway: Some(Ipv4Addr::new(192, 168, 3, 1)),
        dns_servers: Default::default(),
    });

    let mut rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init AP network stack
    use embassy_net::StackResources;
    let (ap_stack, ap_runner) = embassy_net::new(
        wifi_ap_device,
        ap_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );
    // EOF WIFI setup

    // Init communication back and forth channel between udp and uart task
    let udp2ecu_buffer = mk_static!(
        [ChannelItem; CHANNEL_ITEMS_MAX],
        [ChannelItem::empty(); CHANNEL_ITEMS_MAX]
    );
    let udp2ecu_channel = mk_static!(
        Channel<'_, NoopRawMutex, ChannelItem>,
        Channel::new(udp2ecu_buffer)
    );
    let (udp_sender, ecu_receiver) = udp2ecu_channel.split();

    let ecu2udp_buffer = mk_static!(
        [ChannelItem; CHANNEL_ITEMS_MAX],
        [ChannelItem::empty(); CHANNEL_ITEMS_MAX]
    );
    let ecu2udp_channel = mk_static!(
        Channel<'_, NoopRawMutex, ChannelItem>,
        Channel::new(ecu2udp_buffer)
    );
    let (ecu_sender, udp_receiver) = ecu2udp_channel.split();

    // initialize ECU specific adapter
    #[cfg(any(feature = "elm327", feature = "l9637"))]
    let ecu_adapter = adapter::Adapter::new(
        /*
        peripherals.UART0.into(),
        peripherals.GPIO21.into(),
        peripherals.GPIO20.into(),
        */
        peripherals.UART1.into(),
        peripherals.GPIO2.into(),
        peripherals.GPIO3.into(),
    );

    //crate::debug_pin::init_debug_pin(peripherals.GPIO0);

    // spawn tasks
    spawner
        .spawn(udp::server(controller, ap_stack, udp_sender, udp_receiver))
        .ok();

    spawner
        .spawn(ecu::server(ecu_sender, ecu_receiver, ecu_adapter))
        .ok();

    spawner.spawn(net_task(ap_runner)).ok();
    spawner.spawn(run()).ok();
}

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("0");

        embassy_time::Timer::after(embassy_time::Duration::from_millis(1_000)).await;
    }
}

#[embassy_executor::task()]
pub async fn net_task(
    mut runner: embassy_net::Runner<'static, esp_radio::wifi::WifiDevice<'static>>,
) {
    runner.run().await
}
