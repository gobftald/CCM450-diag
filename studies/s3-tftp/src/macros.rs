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
macro_rules! create_ap_sta {
    ($peripherals:ident) => {
        {
            const AP_SSID: Option<&'static str> = option_env!("AP_SSID");
            const AP_GW_IP: Option<&'static str> = option_env!("AP_GW_IP");
            const STA_SSID:  Option<&'static str> = option_env!("TETH_SSID");
            const STA_PWD:  Option<&'static str> = option_env!("TETH_PWD");

            // controller is shared for both AP and STA
            let esp_radio_ctrl = &*mk_static!(
                esp_radio::Controller<'static>,
                unwrap!(esp_radio::init())
            );

            // Single wifi::new() call yields both interfaces
            let (mut controller, interfaces) = unwrap!(esp_radio::wifi::new(
                esp_radio_ctrl,
                $peripherals.WIFI,
                Default::default()
            ));

            let wifi_ap_device = interfaces.ap;
            let wifi_sta_device = interfaces.sta;

            // Configure AP+STA combined mode here while constants are in scope
            let mode_config = esp_radio::wifi::ModeConfig::ApSta(
                        esp_radio::wifi::ClientConfig::default()
                            .with_ssid(unwrap!(STA_SSID).try_into().unwrap())
                            .with_password(unwrap!(STA_PWD).try_into().unwrap()),
                        esp_radio::wifi::AccessPointConfig::default()
                            .with_ssid(unwrap!(AP_SSID).into()),
                );
            unwrap!(controller.set_config(&mode_config));

            // ── AP stack (static IP) ─────────────────────────────────────────────
            use core::{net::Ipv4Addr, str::FromStr};
            let gw_ip_addr = unwrap!(
                Ipv4Addr::from_str(AP_GW_IP.unwrap_or("192.168.2.1")),
                "failed to parse gateway ip"
            );

            let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
                address: embassy_net::Ipv4Cidr::new(gw_ip_addr, 24),
                gateway: Some(gw_ip_addr),
                dns_servers: Default::default(),
            });

            // ── STA stack (DHCP) ─────────────────────────────────────────────────
            let sta_config = embassy_net::Config::dhcpv4(
                embassy_net::DhcpConfig::default()
            );

            let rng = esp_hal::rng::Rng::new();
            let seed1 = (rng.random() as u64) << 32 | rng.random() as u64;
            let seed2 = (rng.random() as u64) << 32 | rng.random() as u64;

            // Init AP network stack
            use embassy_net::StackResources;
            let (ap_stack, ap_runner) = embassy_net::new(
                wifi_ap_device,
                ap_config,
                mk_static!(StackResources<3>, StackResources::<3>::new()),
                seed1,
            );

            // Init STA netwrok task
            let (sta_stack, sta_runner) = embassy_net::new(
                wifi_sta_device,
                sta_config,
                mk_static!(StackResources<3>, StackResources::<3>::new()),
                seed2,
            );


            (controller, ap_runner, ap_stack, sta_runner, sta_stack)
        }
    }
}

#[macro_export]
macro_rules! create_spi_bus {
    ($peripherals:ident) => {{
        use esp_hal::dma::{DmaDescriptor, DmaRxBuf, DmaTxBuf};
        use static_cell::StaticCell;

        const DMA_RX_BUFFER_SIZE: usize = 512;          // SD card block size
        const DMA_TX_BUFFER_SIZE: usize = 32 * 128;     // large for LCD transfers

        const DMA_RX_TX_DESCRIPTORS_SIZE: usize = 2;    // 1 for every 4092 (not 4096, it is HW limitation)

        static RX_DATA: StaticCell<[u8; DMA_RX_BUFFER_SIZE]> = StaticCell::new();
        static TX_DATA: StaticCell<[u8; DMA_TX_BUFFER_SIZE]> = StaticCell::new();

        #[repr(align(32))]
        struct AlignedDmaDescriptors([DmaDescriptor; DMA_RX_TX_DESCRIPTORS_SIZE]);
        static RX_DESCRIPTORS: static_cell::StaticCell<AlignedDmaDescriptors> = static_cell::StaticCell::new();
        static TX_DESCRIPTORS: static_cell::StaticCell<AlignedDmaDescriptors> = static_cell::StaticCell::new();

        let rx_descriptors = RX_DESCRIPTORS.init(AlignedDmaDescriptors([esp_hal::dma::DmaDescriptor::EMPTY; DMA_RX_TX_DESCRIPTORS_SIZE]));
        let tx_descriptors = TX_DESCRIPTORS.init(AlignedDmaDescriptors([esp_hal::dma::DmaDescriptor::EMPTY; DMA_RX_TX_DESCRIPTORS_SIZE]));

        let rx_buf = unwrap!(DmaRxBuf::new(
            &mut rx_descriptors.0,
            RX_DATA.init([0u8; DMA_RX_BUFFER_SIZE])
        ));
        let tx_buf = unwrap!(DmaTxBuf::new(
            &mut tx_descriptors.0,
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
        use esp_hal::i2c::master::{Config, I2c, BusTimeout, SoftwareTimeout};
        use esp_hal::time::Duration;
        unwrap!(
            //I2c::new($peripherals.I2C1, Config::default()),
            I2c::new($peripherals.I2C1,
                Config::default()
                    // There was a warning in touch screen cst816s crate, that "on some devices, attempting
                    // to read registers when there is no data available results in a hang in the i2c read".
                    //
                    // But esp-hal's I2C driver does have timeout mechanisms built in, at multiple levels:
                    // Hardware-level SCL bus timeout (BusTimeout) and Software timeout for I2C operations.
                    // (it needed a patch in esp-hal/src/i2c/master/mod.rs)
                    .with_timeout(BusTimeout::Maximum)
                    .with_software_timeout(SoftwareTimeout::Transaction(Duration::from_millis(50)))
            ),
            "Failed to initialize I2C"
        )
        .with_sda($peripherals.GPIO48)
        .with_scl($peripherals.GPIO47)
        //.into_async()                     // touch screen has a blocking driver
    }};
}
