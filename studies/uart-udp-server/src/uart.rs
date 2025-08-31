use embassy_sync::{blocking_mutex::raw::NoopRawMutex, zerocopy_channel::Sender};
// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;

#[embassy_executor::task()]
pub async fn client(
    mut rx: esp_hal::uart::UartRx<'static>,
    mut sender: Sender<'static, NoopRawMutex, crate::udp::ChannelItem>,
) {
    loop {
        let response_item = sender.send().await;

        // wait for uart response and put it to the channel
        response_item.size = unwrap!(rx.read_async(&mut response_item.data, false).await) as u8;

        debug!(
            "#### uart got {} {}",
            response_item.size,
            &response_item.data[..response_item.size as usize]
        );

        // wake receiver
        sender.send_done();
    }
}
