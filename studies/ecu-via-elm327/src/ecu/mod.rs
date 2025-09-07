#[cfg_attr(feature = "ccm450", path = "ccm450.rs")]
mod ecu;

use embassy_futures::select::{Either, select};
// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};

// arbitrary high-level ECU commands
enum Request {
    Connect,
    Disconnect,
    LiveDataStart,
    LiveDataStop,
}

enum Response {
    Ok,
}

enum Error {}

trait Ecu {
    fn new() -> Self;
    fn connect(&self);
    fn process_request(&self, req: Request) -> Result<Response, Error>;
    fn process_response(&self);
}

#[embassy_executor::task()]
pub async fn server(
    mut sender: Sender<'static, NoopRawMutex, crate::ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, crate::ChannelItem>,

    #[cfg(any(feature = "elm327", feature = "l9637"))] mut adapter: crate::ecu_adapter::Adapter<
        'static,
    >,
) {
    // get a specific ECU
    #[cfg(feature = "ccm450")]
    let ecu = ecu::ECU::new();

    // get a fresh channel item to write to
    let mut response_item = sender.send().await;

    response_item.size = 0;
    loop {
        // waiting for request or response
        match select(
            receiver.receive(),
            adapter.read_async(&mut response_item.data),
        )
        .await
        {
            Either::First(received_item) => {
                trace!("#### ECU: receiver.receive(): size: {}", received_item.size);

                ecu.process_request(Request::Connect);

                unwrap!(
                    adapter
                        .write_async(&mut received_item.data[..received_item.size])
                        .await
                );

                trace!("#### ECU: tx.write_async()");

                receiver.receive_done();
            }

            Either::Second(result) => {
                crate::debug_pin::debug_pin(0);

                response_item.size = result.unwrap_or_else(|_err| {
                    trace!("RxError: {}", _err);
                    0
                });

                // if not error forward response to udp
                if response_item.size > 0 {
                    trace!(
                        "#### ECU: read_async(): size: {} data: {}",
                        response_item.size,
                        &response_item.data[..response_item.size]
                    );

                    ecu.process_response();

                    // wake receiver
                    sender.send_done();

                    // wait channel to clear (it takes until data was sent by udp)
                    response_item = sender.send().await;
                }
            }
        }
    }
}
