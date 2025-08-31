use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};
// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;

use crate::ChannelItem;

#[embassy_executor::task()]
pub async fn client(
    mut sender: Sender<'static, NoopRawMutex, ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, ChannelItem>,
    uart0: esp_hal::peripherals::UART0<'static>,
    tx_pin: esp_hal::peripherals::GPIO20<'static>,
    rx_pin: esp_hal::peripherals::GPIO21<'static>,
) {
    let config = esp_hal::uart::Config::default();
    let uart0 = unwrap!(esp_hal::uart::Uart::new(uart0, config))
        .with_tx(tx_pin)
        .with_rx(rx_pin);
    let (rx, tx) = uart0.split();

    loop {
        let received_item = receiver.receive().await;
        let received_size = received_item.size;

        if let Ok(_s) = core::str::from_utf8(&received_item.data[..received_size as usize]) {
            debug!("ECHO: {}", _s);
        } else {
            debug!("ECHO: bytearray len {}", received_size);
        }

        let sending_item = sender.send().await;

        echo(&*received_item, sending_item);
        //process_udp2ecu_request(&*received_item, sending_item);

        receiver.receive_done();

        sender.send_done();
    }
}

fn echo(request: &ChannelItem, response: &mut ChannelItem) {
    response.data[0] = b'#';
    unsafe {
        core::ptr::copy_nonoverlapping::<u8>(
            &request.data as *const u8,
            &mut response.data[1] as *mut u8,
            request.size as usize,
        );
    }
    response.size = request.size + 1;
}

fn process_udp2ecu_request(request: &ChannelItem, response: &mut ChannelItem) {}
