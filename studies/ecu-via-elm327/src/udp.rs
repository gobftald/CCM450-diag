use embassy_futures::select::{Either, select};
use embassy_net::udp::UdpMetadata;

// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};

pub const UDP_BUFFER_SIZE: usize = crate::CHANNEL_ITEM_SIZE;
const UDP_PACKET_MAX: usize = 4;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(strum_macros::FromRepr)]
pub enum Subsystem {
    // itself
    System,
    Ecu,
    Gps,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Reply {
    InvalidSubystem,
}

#[embassy_executor::task()]
pub async fn server(
    mut controller: esp_radio::wifi::WifiController<'static>,
    ap_stack: embassy_net::Stack<'static>,
    mut uart_sender: Sender<'static, NoopRawMutex, crate::ChannelItem>,
    mut uart_receiver: Receiver<'static, NoopRawMutex, crate::ChannelItem>,
) {
    // config AP and start WiFi
    let client_config =
        esp_radio::wifi::Configuration::AccessPoint(esp_radio::wifi::AccessPointConfiguration {
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
    unwrap!(ap_udp_server_socket.bind(19924));
    let mut end_point: Option<UdpMetadata> = None;

    loop {
        // wait for the channel to clear
        let request_item = uart_sender.send().await;

        // waiting for request or response
        match select(
            ap_udp_server_socket.recv_from(&mut request_item.data),
            uart_receiver.receive(),
        )
        .await
        {
            // UDP request arrived
            Either::First(result) => {
                let (n, ep) = unwrap!(result);

                trace!(
                    "#### UDP: ap_udp_server_socket.recv_from(): {}",
                    request_item.data[..n]
                );

                if let Some(subsystem) = Subsystem::from_repr(request_item.data[0] as usize) {
                    match subsystem {
                        Subsystem::System => {}
                        Subsystem::Ecu => {
                            // send all icoming data
                            request_item.size = n;

                            // forward request to uart
                            uart_sender.send_done();
                        }
                        Subsystem::Gps => {}
                    }
                } else {
                    unwrap!(
                        ap_udp_server_socket
                            // error sending subsystem is the system itself, which is 0 as u8
                            .send_to(&[Subsystem::System as u8, Reply::InvalidSubystem as u8], ep)
                            .await
                    );
                    debug!("#### ap_udp_server_socket.send_to() returned");
                }

                end_point = Some(ep);
            }

            // Uart answer arrived
            Either::Second(received_item) => {
                trace!(
                    "#### UDP: uart_receiver.receive(): size: {}",
                    received_item.size
                );

                // forward response via UDP
                if let Some(end_point) = end_point {
                    unwrap!(
                        ap_udp_server_socket
                            .send_to(&received_item.data[..received_item.size], end_point)
                            .await
                    );
                    debug!("#### ap_udp_server_socket.send_to() returned");
                }

                uart_receiver.receive_done();
            }
        };

        /*
        //needs --features=esp-alloc/internal-heap-stats
        unsafe {
            debug!("{}", esp_alloc::HEAP.stats());
        }
        */
    }
}
