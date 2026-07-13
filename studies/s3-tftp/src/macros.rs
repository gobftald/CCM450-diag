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

            // AP config in a separate variable so that connection_task can
            // reuse it in each connection cycle when compiling ModeConfig::ApSta
            let ap_config_static = esp_radio::wifi::AccessPointConfig::default()
                .with_ssid(env!("AP_SSID").into());

            // Initial mode config -- placeholder with STA client config, this will
            // be overwritten by connection_task immediately after the first scan
            let mode_config = esp_radio::wifi::ModeConfig::ApSta(
                        esp_radio::wifi::ClientConfig::default(),
                            //.with_ssid(env!("TETH_SSID").into())
                            //.with_password(env!("TETH_PWD").into()),
                        //esp_radio::wifi::AccessPointConfig::default()
                            //.with_ssid(env!("AP_SSID").into()),
                        ap_config_static.clone(),
                );
            unwrap!(controller.set_config(&mode_config));

            // ── AP stack (static IP) ─────────────────────────────────────────────
            use core::{net::Ipv4Addr, str::FromStr};
            let gw_ip_addr = unwrap!(
                Ipv4Addr::from_str(env!("AP_GW_IP")),
                "failed to parse gateway ip"
            );

            let ap_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
                address: embassy_net::Ipv4Cidr::new(gw_ip_addr, 24),
                gateway: Some(gw_ip_addr),
                dns_servers: Default::default(),
            });

            // ── STA stack (DHCP) ─────────────────────────────────────────────────
            let sta_config = embassy_net::Config::dhcpv4(embassy_net::DhcpConfig::default());

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


            (controller, ap_runner, ap_stack, sta_runner, sta_stack, ap_config_static)
        }
    }
}

#[macro_export]
macro_rules! create_spi_bus {
    ($peripherals:ident) => {{
        use esp_hal::dma::{DmaRxBuf, DmaTxBuf};
        use esp_hal::dma_buffers;
        use static_cell::StaticCell;

        const DMA_RX_BUFFER_SIZE: usize = 512;          // SD card block size
        const DMA_TX_BUFFER_SIZE: usize = 32 * 128;     // large for LCD transfers

        let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
            dma_buffers!(DMA_RX_BUFFER_SIZE, DMA_TX_BUFFER_SIZE);

        let rx_buf = unwrap!(DmaRxBuf::new(rx_descriptors, rx_buffer));
        let tx_buf = unwrap!(DmaTxBuf::new(tx_descriptors, tx_buffer));

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
