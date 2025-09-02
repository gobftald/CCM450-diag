use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};
// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;

#[embassy_executor::task()]
pub async fn client(
    mut udp_sender: Sender<'static, NoopRawMutex, crate::ChannelItem>,
    mut udp_receiver: Receiver<'static, NoopRawMutex, crate::ChannelItem>,
    uart0: esp_hal::peripherals::UART0<'static>,
    tx_pin: esp_hal::peripherals::GPIO21<'static>,
    rx_pin: esp_hal::peripherals::GPIO20<'static>,
) {
    // configure UART
    let config = esp_hal::uart::Config::default();
    let mut uart0 = unwrap!(esp_hal::uart::Uart::new(uart0, config))
        .with_tx(tx_pin)
        .with_rx(rx_pin);
    uart0.set_at_cmd(esp_hal::uart::AtCmdConfig::default());
    let (mut rx, mut tx) = uart0.split();

    loop {
        // wait for the channel to clear
        let response_item = udp_sender.send().await;

        // waiting for request or response
        match select(
            udp_receiver.receive(),
            rx.read_async(&mut response_item.data, false),
        )
        .await
        {
            Either::First(received_item) => {
                // forward request to uart
                //debug!("#### UART: udp_receiver.receive()");
                unwrap!(
                    tx.write_async(&received_item.data[..received_item.size])
                        .await
                );
                unwrap!(tx.flush_async().await);
                //debug!("#### UART: tx.write_async()");
                udp_receiver.receive_done();
            }

            Either::Second(result) => {
                crate::debug_pin::debug_pin(0);
                // forward response to udp
                response_item.size = result.unwrap();
                /*
                debug!(
                    "#### UART: rx.read_async(): size: {} daTa: {}",
                    response_item.size,
                    &response_item.data[..response_item.size]
                );
                */

                // wake receiver
                udp_sender.send_done();
            }
        };
    }
}
