#![allow(static_mut_refs)]

#![no_std]
#![no_main]
// for 'task' embassy-executor-macro, when embassy-executor/nightly
#![feature(impl_trait_in_assoc_type)]
// for 'waiti 0' instructions in panic handler
#![cfg_attr(target_arch = "xtensa", feature(asm_experimental_arch))]

pub(crate) mod fmt;

use core::{net::Ipv4Addr, str::FromStr};

use esp_hal::{
    gpio::{Input, Output, InputConfig, OutputConfig, DriveMode, Level, Pull},
    rtc_cntl::{Rtc, sleep::{TimerWakeupSource, Ext0WakeupSource, WakeupLevel}, reset_reason, wakeup_cause},
    system::SleepSource,
};
use embassy_time::Timer;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};


mod macros;
mod panic;

mod gps;
mod gpx;
mod gsm;
mod imu;
mod lcd;
mod sd_card;
mod tcp;
mod tftp;

#[cfg(not(feature = "rtt"))]
use esp_println as _;           // if no "rtt-target/defmt" we need "esp-println/defmt-espflash" in "dfmt"

// for consuming bootlader RAM segment for heap
esp_bootloader_esp_idf::esp_app_desc!();

type SharedSpiBus = embassy_sync::mutex::Mutex<
    NoopRawMutex, esp_hal::spi::master::SpiDmaBus<'static, esp_hal::Async>
>;

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
    //esp_alloc::heap_allocator!(size: 32 * 1024);

    //esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    // debug version of the original macro
    init_psram_heap!(peripherals.PSRAM);

    esp_rtos_start!(peripherals);

    match wakeup_cause() {
        SleepSource::Ext0 => {
            Timer::after_millis(2_000).await;
            debug!("awakening from deep sleep caused by motion");
        }
        SleepSource::Timer => {
            Timer::after_millis(2_000).await;
            debug!("awakening from deep sleep casued by timer");
        }
        _ => {}
    }

    let (
        controller,
        ap_runner,
        ap_stack,
        sta_runner,
        sta_stack,
        ap_config,
    ) = create_ap_sta!(peripherals);

    // they cannot be simple static, since NoopRawMutex does not Sync
    // but StaticCell<T> defined as Sync even if T is not Sync
    // we can use my SyncNoopRawMutex from gps.rs but this way is simpler
    let wifi_rescan_request = mk_static!(
        Signal<NoopRawMutex, ()>,
        Signal::<NoopRawMutex, ()>::new()
    );
    let sta_state_changed = mk_static!(
        Signal<NoopRawMutex, ()>,
        Signal::<NoopRawMutex, ()>::new()
    );

    // We should move out all created type (controller, ap_runner, ap_stack senders and receivers
    // from this main/loader task. Although finally it/we exit(s), spawns below (which finally 
    // conclude to 'await's) would force task's state machine to keep these value in its memory,
    // increasing the wasted memory footprint of this exited so practically zombie task.
    //
    // Because of the compiler optimisation even we should not only move out but also
    // should use these types handed over by value in the spawned tasks, otherwise they
    // are also staying and increasing the wasted memory footprint of exited main task
    let _ = spawner.spawn(ap_net_task(ap_runner));
    let _ = spawner.spawn(dhcp_task(ap_stack));

    let _ = spawner.spawn(sta_net_task(sta_runner));
    let _ = spawner.spawn(tcp::connection_task(
        controller, ap_config, sta_stack, wifi_rescan_request, sta_state_changed
    ));
    let _ = spawner.spawn(tcp::tcp_task(sta_stack, sta_state_changed));

    let spi = create_spi_bus!(peripherals);
    let mut i2c = create_i2c_bus!(peripherals);

    // pin# based on WaveShare ESP32-S3-Touch-LCD-2 schematic

    // chip select pins of all spi periherals should be initialized before creating shared spi bus
    let sd_cs = Output::new(peripherals.GPIO41, Level::High, OutputConfig::default());
    let lcd_cs = Output::new(peripherals.GPIO45, Level::High, OutputConfig::default());

    let (proxy, storage) = sd_card::init(spi, sd_cs).await;
    let _ = spawner.spawn(sd_card::sd_task(proxy, storage));

    let _ = spawner.spawn(tftp::tftp_task(sta_stack, storage, proxy));

    let _ = spawner.spawn(gps::gps_task(create_gps_uart!(peripherals)));

    let (gsm_uart, pwk) = create_gsm_uart!(peripherals);
    let _ = spawner.spawn(gsm::gsm_task(gsm_uart, pwk));

    let _ = spawner.spawn(system_stats());

    lcd::lcd_task(
        &mut i2c,
        Input::new(
            peripherals.GPIO46,
            InputConfig::default().with_pull(Pull::None),  // TP INT
        ),
        spi,
        wifi_rescan_request,
        lcd_cs,
        peripherals.GPIO42.into(),  // LCD DC
        peripherals.GPIO0.into(),   // LCD RST
        peripherals.GPIO1.into(),   // LCD BL
    ).await;

    // init motion detector then go to deep sleep

    let mut imu_int = peripherals.GPIO3;
    let imu_int_input = Input::new(
            imu_int.reborrow(),
            InputConfig::default().with_pull(Pull::None) // IMU INT
    );

    let _ = imu::imu_init(&mut i2c);

    let mut rtc = Rtc::new(peripherals.LPWR);
    let timer = TimerWakeupSource::new(core::time::Duration::from_secs(30));
    core::mem::drop(imu_int_input);
    let ext0 = Ext0WakeupSource::new(imu_int, WakeupLevel::High);

    rtc.sleep_deep(&[&timer, &ext0]);
}

#[embassy_executor::task]
async fn system_stats() {
    #[cfg(not(feature = "idle_stats"))]
    let mut counter = 0;

    #[cfg(feature = "heap_stats")]
    let mut current_heap_usage = 0;

    cfg_if::cfg_if! {
        if #[cfg(feature = "idle_stats")]
        {
            use esp_hal::time::Instant;

            let mut idle_prev: esp_hal::time::Duration = esp_hal::time::Duration::ZERO;
            let mut prev_time_stamp: Instant = Instant::now();
        }
    }

    loop {
        #[cfg(feature = "heap_stats")]
        {
            let heap_stats = esp_alloc::HEAP.stats();

            if heap_stats.current_usage != current_heap_usage {
                info!("{}", heap_stats);
                current_heap_usage = heap_stats.current_usage;
            }
        }

        cfg_if::cfg_if! {
            if #[cfg(any(feature = "idle_stats", feature = "irq_stats"))]
            {
                #[cfg(feature = "idle_stats")]
                {
                    let time_stamp = Instant::now();
                    let idle_current = (esp_rtos::idle_stats() - idle_prev).as_millis() as f32
                        / (time_stamp - prev_time_stamp).as_millis() as f32 * 100f32;
                    info!("idle: {}.{:02}%", idle_current as i32, (idle_current * 100.0) as i32 % 100 );

                    // don't include the division and formatting time in the measurement
                    idle_prev = esp_rtos::idle_stats();
                    prev_time_stamp = time_stamp;
                }
                
                #[cfg(feature = "irq_stats")]
                {
                    let irq_stats = esp_hal::interrupt::irq_stats();
                    info!("irq: {}", irq_stats.1[0..irq_stats.0]);
                }
            } else {
                //esp_println::println!("{}", counter);
                info!("{}", counter);
                counter += 1;
            }
        }

        Timer::after_millis(1_000).await;
    }
}

#[embassy_executor::task]
pub async fn ap_net_task(
    mut runner: embassy_net::Runner<'static, esp_radio::wifi::WifiDevice<'static>>,
) {
    runner.run().await
}

#[embassy_executor::task]
pub async fn sta_net_task(
    mut runner: embassy_net::Runner<'static, esp_radio::wifi::WifiDevice<'static>>,
) {
    runner.run().await
}

#[embassy_executor::task]
async fn dhcp_task(ap_stack: embassy_net::Stack<'static>) {
    use leasehund::DhcpServer;

    let mut server: DhcpServer<4, 1> = DhcpServer::new(
        unwrap!(Ipv4Addr::from_str(env!("AP_GW_IP"))),
        unwrap!(Ipv4Addr::from_str(env!("AP_DHCP_SUBNET"))),
        unwrap!(Ipv4Addr::from_str(env!("AP_GW_IP"))),
        unwrap!(Ipv4Addr::from_str(env!("AP_GW_IP"))),
        unwrap!(Ipv4Addr::from_str(env!("AP_DHCP_POOL_START"))),
        unwrap!(Ipv4Addr::from_str(env!("AP_DHCP_POOL_END"))),
    );
    server.run(ap_stack).await;
}

/*
Why after_ticks(1) is better than yield_now() here:
yield_now(): Puts you at the back of the line. If no other task is "ready," you start again immediately.
after_ticks(1): Actually suspends the task for one hardware timer tick. This gives the CPU a guaranteed
"breather" to handle any pending interrupts from your UART or Buttons without the SD task immediately
trying to hog the SPI bus again.
*/

/*
Preventing UART OverflowsIf your UART is high-speed (like 115200+), even a single sector read (512 bytes)
can take long enough to overflow a small hardware FIFO.To solve this, ensure your UART Task is running at
a higher priority than your SD Task. In Embassy, you can do this by using multiple executors or, more simply,
by using Interrupt-driven UART
*/
