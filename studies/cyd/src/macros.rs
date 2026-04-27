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
macro_rules! create_channels {
    (TYPE $ChannelItem:ident, $ChannelItemSize:ident, $SizeValue:expr, $ChannelItemMax:ident, $MaxValue:expr) => {

        pub const $ChannelItemSize: usize = $SizeValue;
        pub const $ChannelItemMax: usize = $MaxValue;

        #[derive(Clone, Copy)]
        pub struct $ChannelItem {
            pub size: usize,
            pub data: [u8; $ChannelItemSize],
        }

        impl $ChannelItem {
            const fn empty() -> Self {
                Self {
                    size: 0,
                    data: [0; $ChannelItemSize],
                }
            }
        }
    };

    (INIT $ChannelItem:ident, $ChannelItemMax:ident) => {{
        // we can use NoopRawMutex since we use channel between two tasks in the same executor,
        // in single core environment and not using from interrupt
        // the more future-proof ThreadModeRawMutex is implemented only for cortex_m in embassy_sync
        // and CriticalSectionRawMutex is unecessary for this case
        use embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel};

        // Init communication back and forth channel between udp and uart task
        let udp2ecu_buffer = mk_static!(
            [$ChannelItem; $ChannelItemMax],
            [$ChannelItem::empty(); $ChannelItemMax]
        );
        let udp2ecu_channel = mk_static!(
            Channel<'_, NoopRawMutex, ChannelItem>,
            Channel::new(udp2ecu_buffer)
        );
        let (udp_tx, ecu_rx) = udp2ecu_channel.split();

        let ecu2udp_buffer = mk_static!(
            [$ChannelItem; $ChannelItemMax],
            [$ChannelItem::empty(); $ChannelItemMax]
        );
        let ecu2udp_channel = mk_static!(
            Channel<'_, NoopRawMutex, ChannelItem>,
            Channel::new(ecu2udp_buffer)
        );
        let (ecu_tx, udp_rx) = ecu2udp_channel.split();

        (udp_tx, udp_rx, ecu_tx, ecu_rx)
    }}
}