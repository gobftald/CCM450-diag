#[cfg_attr(feature = "ccm450", path = "ccm450.rs")]
mod ecu_implementation;
use ecu_implementation::ECU;

use embassy_futures::select::{Either, select};
// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};

// arbitrary high-level ECU commands#[]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(strum_macros::FromRepr)]
enum Request {
    Connect,
    Disconnect,
    LiveDataStart,
    LiveDataStop,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum EcuError {
    InvalidRequest,
    CommunicationFailed,
}

trait Ecus {
    async fn request(&mut self, request: Request) -> Result<(), EcuError>;
    async fn response(&mut self, response: &mut [u8]) -> Result<usize, EcuError>;
}

#[embassy_executor::task()]
pub async fn server(
    mut sender: Sender<'static, NoopRawMutex, crate::ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, crate::ChannelItem>,

    #[cfg(any(feature = "elm327", feature = "l9637"))] adapter: crate::ecu_adapter::Adapter<
        'static,
    >,
) {
    // get a specific ECU
    #[cfg(feature = "ccm450")]
    let mut ecu = ECU::new(adapter);

    // get a fresh channel item to write to
    let mut response_item = sender.send().await;

    response_item.size = 0;
    loop {
        trace!(
            "ecu/mod: match select(receiver.receive(), ecu.response(&mut response_item.data)).await"
        );
        // waiting for request or response
        match select(receiver.receive(), ecu.response(&mut response_item.data)).await {
            // received an ecu request
            Either::First(received_item) => {
                trace!(
                    "#### ECU: received {}",
                    received_item.data[..received_item.size]
                );

                if received_item.size == 1
                    && let Some(request) = Request::from_repr(received_item.data[0] as usize)
                {
                    ecu.request(request)
                        .await
                        .unwrap_or_else(|err| error!("{}", err));
                }

                // we have finished to process request
                receiver.receive_done();
            }

            // received responss from ecu
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

                    // wake receiver
                    sender.send_done();

                    // wait channel to clear (it takes until data was sent by udp)
                    response_item = sender.send().await;
                }
            }
        }
    }
}
