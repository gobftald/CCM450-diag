use embassy_futures::select::{Either, select};

// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};

use crate::ChannelItem;

use esp_hal::{gpio::AnyPin, uart::AnyUart};

#[embassy_executor::task()]
pub async fn client(
    mut udp_sender: Sender<'static, NoopRawMutex, crate::ChannelItem>,
    mut udp_receiver: Receiver<'static, NoopRawMutex, crate::ChannelItem>,
    uart: AnyUart<'static>,
    tx_pin: AnyPin<'static>,
    rx_pin: AnyPin<'static>,
) {
    // configure UART
    let config = esp_hal::uart::Config::default().with_baudrate(38_400);
    let mut uart = unwrap!(esp_hal::uart::Uart::new(uart, config))
        .with_tx(tx_pin)
        .with_rx(rx_pin);
    uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default());
    let (mut rx, mut tx) = uart.split();

    // buffer for read_async
    let mut response_buffer: [u8; size_of::<ChannelItem>()] = [0; size_of::<ChannelItem>()];

    // get a fresh channel item to write to
    let mut response_item = udp_sender.send().await;

    response_item.size = 0;
    loop {
        // waiting for request or response
        match select(
            udp_receiver.receive(),
            rx.read_async(&mut response_buffer, false),
        )
        .await
        {
            Either::First(received_item) => {
                trace!(
                    "#### UART: udp_receiver.receive(): size: {}",
                    received_item.size
                );

                // forward request to uart
                unwrap!(
                    tx.write_async(&received_item.data[..received_item.size])
                        .await
                );
                unwrap!(tx.flush_async().await);
                trace!("#### UART: tx.write_async()");

                udp_receiver.receive_done();
            }

            Either::Second(result) => {
                crate::debug_pin::debug_pin(0);

                // forward response to udp
                let response_size = result.unwrap_or_else(|_err| {
                    trace!("RxError: {}", _err);
                    0
                });
                if response_size > 0 {
                    trace!(
                        "#### UART: rx.read_async(): size: {} data: {}",
                        response_size,
                        &response_buffer[..response_size]
                    );

                    let mut cr = false;
                    for data in response_buffer[..response_size].iter() {
                        // drop if not ascii
                        if *data < 128 {
                            trace!(
                                "data processed: data: {} index: {}",
                                *data, response_item.size
                            );

                            // store response into channel
                            response_item.data[response_item.size] = *data;
                            response_item.size += 1;

                            if *data == b'\r' {
                                cr = true;
                                break;
                            }
                            // don't forward until we've read all incoming data from rx fifo
                        }
                    }

                    // forwarding uart response to udp only if it contains carriage return
                    // but carriage retunr is not the first
                    if cr {
                        // wake receiver
                        udp_sender.send_done();

                        // wait channel to clear (it takes until data was sent by udp)
                        response_item = udp_sender.send().await;
                        response_item.size = 0;
                    }
                }
            }
        };
    }
}
