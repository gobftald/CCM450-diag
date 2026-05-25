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
            /*
            let gpio39_lcd = unsafe { $peripherals.GPIO39.clone_unchecked() };
            let gpio38_lcd = unsafe { $peripherals.GPIO38.clone_unchecked() };
            */

            const DMA_BUFFER_SIZE: usize = 520;
            use esp_hal::dma::{DmaDescriptor, DmaRxBuf, DmaTxBuf};
            use static_cell::StaticCell;

            static SD_RX_DATA: StaticCell<[u8; DMA_BUFFER_SIZE]> = StaticCell::new();
            static SD_TX_DATA: StaticCell<[u8; DMA_BUFFER_SIZE]> = StaticCell::new();
            static SD_RX_DESCRIPTORS: StaticCell<[DmaDescriptor; 1]> = StaticCell::new();
            static SD_TX_DESCRIPTORS: StaticCell<[DmaDescriptor; 1]> = StaticCell::new();
            let sd_rx_buf = unwrap!(DmaRxBuf::new(
                SD_RX_DESCRIPTORS.init([DmaDescriptor::EMPTY; 1]),
                SD_RX_DATA.init([0u8; DMA_BUFFER_SIZE])
            ));
            let sd_tx_buf = unwrap!(DmaTxBuf::new(
                SD_TX_DESCRIPTORS.init([DmaDescriptor::EMPTY; 1]),
                SD_TX_DATA.init([0u8; DMA_BUFFER_SIZE])
            ));

            let spi2 = unwrap!(
                esp_hal::spi::master::Spi::new(
                    $peripherals.SPI2,
                    esp_hal::spi::master::Config::default()
                        // speed for init must be <= 400 kHz
                        .with_frequency(esp_hal::time::Rate::from_khz(400))
                        .with_mode(esp_hal::spi::Mode::_0)
                ),
                "Failed to initialize SPI2"
            )
            .with_sck($peripherals.GPIO39)
            .with_mosi($peripherals.GPIO38)
            .with_miso($peripherals.GPIO40)
            .with_dma($peripherals.DMA_CH0)
            .with_buffers(sd_rx_buf, sd_tx_buf)
            .into_async();

            /*
            // SPI3 - LCD, async DMA, 40MHz
            //const DMA_BUFFER_SIZE: usize = 2048;
            //use esp_hal::dma::{DmaDescriptor, DmaRxBuf, DmaTxBuf};
            //use static_cell::StaticCell;

            static LCD_RX_DATA: StaticCell<[u8; DMA_BUFFER_SIZE]> = StaticCell::new();
            static LCD_TX_DATA: StaticCell<[u8; DMA_BUFFER_SIZE]> = StaticCell::new();

            static LCD_RX_DESCRIPTORS: StaticCell<[DmaDescriptor; 1]> = StaticCell::new();
            static LCD_TX_DESCRIPTORS: StaticCell<[DmaDescriptor; 1]> = StaticCell::new();

            let lcd_rx_buf = unwrap!(DmaRxBuf::new(
                LCD_RX_DESCRIPTORS.init([DmaDescriptor::EMPTY; 1]),
                LCD_RX_DATA.init([0u8; DMA_BUFFER_SIZE])
            ));

            let lcd_tx_buf = unwrap!(DmaTxBuf::new(
                LCD_TX_DESCRIPTORS.init([DmaDescriptor::EMPTY; 1]),
                LCD_TX_DATA.init([0u8; DMA_BUFFER_SIZE])
            ));

            let spi3 = unwrap!(
                esp_hal::spi::master::Spi::new(
                    $peripherals.SPI3,
                    esp_hal::spi::master::Config::default()
                        .with_frequency(esp_hal::time::Rate::from_mhz(40))
                        .with_mode(esp_hal::spi::Mode::_0)
                ),
                "Failed to initialize SPI3"
            )
            .with_sck(gpio39_lcd)            // clone used for SPI3
            .with_mosi(gpio38_lcd)
            .with_dma($peripherals.DMA_CH1)
            .with_buffers(lcd_rx_buf, lcd_tx_buf)
            .into_async();
            */

            static CELL: StaticCell<SharedSpiBus> = StaticCell::new();
            CELL.init(SharedSpiBus::new(spi2))
            
            //(CELL.init(SharedSpiBus::new(spi2)), spi3)
    }};
}
