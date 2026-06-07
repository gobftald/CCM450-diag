#[macro_export]
macro_rules! init_psram_heap {
    ($psram:expr) => {{
        let (start, size) = esp_hal::psram::psram_raw_parts(&$psram);
        info!("PSRAM start: {}, size: {} KB", start, size / 1024);
        unsafe {
            esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
                start,
                size,
                esp_alloc::MemoryCapability::External.into(),
            ));
        }
    }};
}

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }}
}

#[macro_export]
macro_rules! esp_rtos_start {
    ($peripherals:ident) => {
        #[cfg(target_arch = "riscv32")]
        use esp_hal::interrupt::software::SoftwareInterruptControl;

        let timg0 = esp_hal::timer::timg::TimerGroup::new($peripherals.TIMG0);

        #[cfg(target_arch = "riscv32")]
        let sw_int = SoftwareInterruptControl::new($peripherals.SW_INTERRUPT);

        extern "C" fn idle_hook() -> ! {
            loop {
                esp_rtos::CurrentThreadHandle::get()
                    .delay(esp_hal::time::Duration::from_millis(400));
            }
        }

        // it should be placed after heap allocation, since esp-rtos
        // main task will allocate all remaining memory
        esp_rtos::start(
            timg0.timer0,
            #[cfg(target_arch = "riscv32")]
            sw_int.software_interrupt0,
        );

        #[cfg(all(feature = "irq_stats", target_arch = "xtensa"))]
        esp_hal::interrupt::register_cpu_interrupt_stat(7);
    }
}

#[macro_export]
macro_rules! create_access_point {
    ($peripherals:ident) => {
        {
            const SSID: Option<&'static str> = option_env!("SSID");
            const GATEWAY_IP: Option<&'static str> = option_env!("GATEWAY_IP");

            let esp_radio_ctrl = &*mk_static!(esp_radio::Controller<'static>, unwrap!(esp_radio::init()));

            let (mut controller, interfaces) = unwrap!(esp_radio::wifi::new(
                &esp_radio_ctrl,
                $peripherals.WIFI,
                Default::default()
            ));

            let client_config =
                esp_radio::wifi::ModeConfig::AccessPoint(
                        esp_radio::wifi::AccessPointConfig::default().with_ssid(unwrap!(SSID).into())
                );
            unwrap!(controller.set_config(&client_config));

            let wifi_ap_device = interfaces.ap;

            use core::{net::Ipv4Addr, str::FromStr};
            let gw_ip_addr = unwrap!(
                Ipv4Addr::from_str(GATEWAY_IP.unwrap_or("192.168.2.1")),
                "failed to parse gateway ip"
            );

            let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
                address: embassy_net::Ipv4Cidr::new(gw_ip_addr, 24),
                gateway: Some(gw_ip_addr),
                dns_servers: Default::default(),
            });

            let rng = esp_hal::rng::Rng::new();
            let seed = (rng.random() as u64) << 32 | rng.random() as u64;

            // Init AP network stack
            use embassy_net::StackResources;
            let (ap_stack, ap_runner) = embassy_net::new(
                wifi_ap_device,
                ap_config,
                mk_static!(StackResources<3>, StackResources::<3>::new()),
                seed,
            );

            (controller, ap_runner, ap_stack)
        }
    }
}

#[macro_export]
macro_rules! create_spi_bus {
    ($peripherals:ident) => {{
        use esp_hal::dma::{DmaDescriptor, DmaRxBuf, DmaTxBuf};
        use static_cell::StaticCell;

        const DMA_RX_BUFFER_SIZE: usize = 512;          // SD card block size
        const DMA_TX_BUFFER_SIZE: usize = 4092;         // large for LCD transfers
        const DMA_RX_TX_DESCRIPTORS_SIZE: usize = 1;    // 1 for every 4092 (not 4096, it is HW limitation)

        // etiher buffers and descriptors are safe in SRAM (not PSRAM)
        static RX_DATA: StaticCell<[u8; DMA_RX_BUFFER_SIZE]> = StaticCell::new();
        static TX_DATA: StaticCell<[u8; DMA_TX_BUFFER_SIZE]> = StaticCell::new();
        static RX_DESCRIPTORS: StaticCell<[DmaDescriptor; DMA_RX_TX_DESCRIPTORS_SIZE]> = StaticCell::new();
        static TX_DESCRIPTORS: StaticCell<[DmaDescriptor; DMA_RX_TX_DESCRIPTORS_SIZE]> = StaticCell::new();
        let rx_buf = unwrap!(DmaRxBuf::new(
            RX_DESCRIPTORS.init([DmaDescriptor::EMPTY; DMA_RX_TX_DESCRIPTORS_SIZE]),
            RX_DATA.init([0u8; DMA_RX_BUFFER_SIZE])
        ));
        let tx_buf = unwrap!(DmaTxBuf::new(
            TX_DESCRIPTORS.init([DmaDescriptor::EMPTY; DMA_RX_TX_DESCRIPTORS_SIZE]),
            TX_DATA.init([0u8; DMA_TX_BUFFER_SIZE])
        ));

        let spi = unwrap!(
            esp_hal::spi::master::Spi::new(
                $peripherals.SPI2,
                esp_hal::spi::master::Config::default()
                    // speed for init must be <= 400 kHz
                    .with_frequency(esp_hal::time::Rate::from_khz(400))
                    .with_mode(esp_hal::spi::Mode::_0)
            ),
            "Failed to initialize SPI"
        )
        .with_sck($peripherals.GPIO39)
        .with_mosi($peripherals.GPIO38)
        .with_miso($peripherals.GPIO40)
        .with_dma($peripherals.DMA_CH0)
        .with_buffers(rx_buf, tx_buf)
        .into_async();

        static CELL: StaticCell<SharedSpiBus> = StaticCell::new();
        CELL.init(SharedSpiBus::new(spi))
    }};
}

#[macro_export]
macro_rules! create_i2c_bus {
    ($peripherals:ident) => {{
        use esp_hal::i2c::master::{Config, I2c};
        unwrap!(
            I2c::new($peripherals.I2C0, Config::default()),
            "Failed to initialize I2C"
        )
        .with_sda($peripherals.GPIO48)
        .with_scl($peripherals.GPIO47)
        //.into_async()
    }};
}
