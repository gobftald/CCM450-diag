use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_sync::zerocopy_channel::{Receiver, Sender};
use esp_hal::sync::RawMutex;

use crate::ChannelItem;

const UDP_BUFFER_SIZE: usize = crate::CHANNEL_ITEM_SIZE;
const UDP_PACKET_MAX: usize = 4;

#[embassy_executor::task()]
pub async fn server(
    mut controller: esp_wifi::wifi::WifiController<'static>,
    ap_stack: embassy_net::Stack<'static>,
    mut sender: Sender<'static, RawMutex, ChannelItem>,
    mut receiver: Receiver<'static, RawMutex, ChannelItem>,
) {
    let client_config =
        esp_wifi::wifi::Configuration::AccessPoint(esp_wifi::wifi::AccessPointConfiguration {
            ssid: "CCM-GP450".into(),
            ..Default::default()
        });
    unwrap!(controller.set_configuration(&client_config));

    debug!("Starting wifi");
    unwrap!(controller.start_async().await);
    debug!("AP started");

    let mut ap_udp_server_rx_meta = [PacketMetadata::EMPTY; UDP_PACKET_MAX];
    let mut ap_udp_server_tx_meta = [PacketMetadata::EMPTY; UDP_PACKET_MAX];
    let mut ap_udp_server_rx_buffer = [0; UDP_BUFFER_SIZE * UDP_PACKET_MAX];
    let mut ap_udp_server_tx_buffer = [0; UDP_BUFFER_SIZE * UDP_PACKET_MAX];

    let mut ap_udp_server_socket = UdpSocket::new(
        ap_stack,
        &mut ap_udp_server_rx_meta,
        &mut ap_udp_server_rx_buffer,
        &mut ap_udp_server_tx_meta,
        &mut ap_udp_server_tx_buffer,
    );

    ap_udp_server_socket.bind(19924).unwrap();

    loop {
        let sending_item = sender.send().await;
        let (n, ep) = ap_udp_server_socket
            .recv_from(&mut sending_item.data)
            .await
            .unwrap();
        sending_item.size = n as u8;
        sender.send_done();

        let received_item = receiver.receive().await;
        ap_udp_server_socket
            .send_to(&received_item.data[..received_item.size as usize], ep)
            .await
            .unwrap();
        receiver.receive_done();

        /*
        //needs --features=esp-alloc/internal-heap-stats
        unsafe {
            debug!("{}", esp_alloc::HEAP.stats());
        }
        */
    }
}
