// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Channel};

const CHANNEL_ITEM_SIZE: usize = 64;
const CHANNEL_ITEMS_MAX: usize = 1;

pub const UDP_BUFFER_SIZE: usize = CHANNEL_ITEM_SIZE;
const UDP_PACKET_MAX: usize = 4;

#[derive(Clone, Copy)]
pub struct ChannelItem {
    pub size: u8,
    pub data: [u8; CHANNEL_ITEM_SIZE - size_of::<u8>()],
}

impl ChannelItem {
    const fn empty() -> Self {
        Self {
            size: 0,
            data: [0; CHANNEL_ITEM_SIZE - size_of::<u8>()],
        }
    }
}

#[embassy_executor::task()]
pub async fn server(
    spawner: embassy_executor::Spawner,
    mut controller: esp_wifi::wifi::WifiController<'static>,
    ap_stack: embassy_net::Stack<'static>,
    uart0: esp_hal::peripherals::UART0<'static>,
    tx_pin: esp_hal::peripherals::GPIO21<'static>,
    rx_pin: esp_hal::peripherals::GPIO20<'static>,
) {
    // config AP and start WiFi
    let client_config =
        esp_wifi::wifi::Configuration::AccessPoint(esp_wifi::wifi::AccessPointConfiguration {
            ssid: "CCM-GP450".into(),
            ..Default::default()
        });
    unwrap!(controller.set_configuration(&client_config));

    debug!("Starting wifi");
    unwrap!(controller.start_async().await);
    debug!("AP started");

    // setup UDP server socket
    use embassy_net::udp::PacketMetadata;

    let mut ap_udp_server_rx_meta = [PacketMetadata::EMPTY; UDP_PACKET_MAX];
    let mut ap_udp_server_tx_meta = [PacketMetadata::EMPTY; UDP_PACKET_MAX];
    let mut ap_udp_server_rx_buffer = [0; UDP_BUFFER_SIZE * UDP_PACKET_MAX];
    let mut ap_udp_server_tx_buffer = [0; UDP_BUFFER_SIZE * UDP_PACKET_MAX];

    let mut ap_udp_server_socket = embassy_net::udp::UdpSocket::new(
        ap_stack,
        &mut ap_udp_server_rx_meta,
        &mut ap_udp_server_rx_buffer,
        &mut ap_udp_server_tx_meta,
        &mut ap_udp_server_tx_buffer,
    );
    ap_udp_server_socket.bind(19924).unwrap();

    // configure UART
    let config = esp_hal::uart::Config::default();
    let mut uart0 = unwrap!(esp_hal::uart::Uart::new(uart0, config))
        .with_tx(tx_pin)
        .with_rx(rx_pin);
    uart0.set_at_cmd(esp_hal::uart::AtCmdConfig::default());
    let (rx, mut tx) = uart0.split();

    // Channel Setup for communication with UART
    use crate::mk_static;
    use core::cell::OnceCell;

    let uart2udp_buffer = mk_static!(
        [ChannelItem; CHANNEL_ITEMS_MAX],
        [ChannelItem::empty(); CHANNEL_ITEMS_MAX]
    );
    let uart2udp_channel = mk_static!(
        Channel<'_, NoopRawMutex, ChannelItem>,
        Channel::new(uart2udp_buffer)
    );
    let (sender, mut receiver) = uart2udp_channel.split();

    // spawn UART client task
    spawner.spawn(crate::uart::client(rx, sender)).ok();

    let mut buf: [u8; UDP_BUFFER_SIZE] = [0; UDP_BUFFER_SIZE];
    loop {
        // waiting for request
        let (n, ep) = ap_udp_server_socket.recv_from(&mut buf).await.unwrap();

        unwrap!(tx.write_async(&buf[..n]).await);
        unwrap!(tx.flush_async().await);

        // waiting for uart response
        let received_item = receiver.receive().await;

        // sending response
        ap_udp_server_socket
            .send_to(&received_item.data[..received_item.size as usize], ep)
            .await
            .unwrap();

        receiver.receive_done();

        //needs --features=esp-alloc/internal-heap-stats
        unsafe {
            debug!("{}", esp_alloc::HEAP.stats());
        }
    }
}
