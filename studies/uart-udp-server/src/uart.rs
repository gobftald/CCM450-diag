use core::ptr::copy_nonoverlapping;

use embassy_sync::zerocopy_channel::{Receiver, Sender};
use esp_hal::sync::RawMutex;

use crate::ChannelItem;

#[embassy_executor::task()]
pub async fn client(
    mut sender: Sender<'static, RawMutex, ChannelItem>,
    mut receiver: Receiver<'static, RawMutex, ChannelItem>,
) {
    loop {
        let received_item = receiver.receive().await;
        let received_size = received_item.size;
        let received_data = &received_item.data as *const u8;
        if let Ok(_s) = core::str::from_utf8(&received_item.data[..received_size as usize]) {
            debug!("ECHO: {}", _s);
        } else {
            debug!("ECHO: bytearray len {}", received_size);
        }
        receiver.receive_done();

        let sending_item = sender.send().await;
        unsafe {
            copy_nonoverlapping::<u8>(
                received_data,
                &mut sending_item.data as *mut u8,
                received_size as usize,
            );
        }
        sending_item.size = received_size;
        sender.send_done();
    }
}
