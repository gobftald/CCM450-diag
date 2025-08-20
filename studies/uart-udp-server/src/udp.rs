use embassy_net::udp::{PacketMetadata, UdpSocket};

#[embassy_executor::task()]
pub async fn server(
    mut controller: esp_wifi::wifi::WifiController<'static>,
    ap_stack: embassy_net::Stack<'static>,
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

    let mut ap_udp_server_rx_meta = [PacketMetadata::EMPTY; 4];
    let mut ap_udp_server_tx_meta = [PacketMetadata::EMPTY; 4];
    let mut ap_udp_server_rx_buffer = [0; 64];
    let mut ap_udp_server_tx_buffer = [0; 64];
    let mut buf = [0; 64];

    let mut ap_udp_server_socket = UdpSocket::new(
        ap_stack,
        &mut ap_udp_server_rx_meta,
        &mut ap_udp_server_rx_buffer,
        &mut ap_udp_server_tx_meta,
        &mut ap_udp_server_tx_buffer,
    );

    ap_udp_server_socket.bind(19924).unwrap();

    loop {
        let (n, ep) = ap_udp_server_socket.recv_from(&mut buf).await.unwrap();
        if let Ok(_s) = core::str::from_utf8(&buf[..n]) {
            debug!("ECHO (to {}): {}", ep, _s);
        } else {
            debug!("ECHO (to {}): bytearray len {}", ep, n);
        }
        ap_udp_server_socket.send_to(&buf[..n], ep).await.unwrap();

        /*
        //needs --features=esp-alloc/internal-heap-stats
        unsafe {
            debug!("{}", esp_alloc::HEAP.stats());
        }
        */
    }
}

#[embassy_executor::task()]
pub async fn net_task(
    mut runner: embassy_net::Runner<'static, esp_wifi::wifi::WifiDevice<'static>>,
) {
    runner.run().await
}
