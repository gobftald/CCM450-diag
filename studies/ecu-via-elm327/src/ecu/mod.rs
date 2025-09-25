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

use crate::{ChannelItem, ecu_adapter::AdapterError};

// arbitrary high-level ECU commands#[]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(strum_macros::FromRepr)]
enum Request {
    Connect,
    Disconnect,
    LiveDataStart,
    LiveDataStop,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum EcuError {
    Ok,
    InvalidRequest,
    InconsystentRequest,
    CommunicationError(AdapterError),
}

trait Ecu {
    async fn connect(&mut self) -> Result<(), EcuError>;
    async fn reply(&mut self, reply: &mut [u8]) -> Result<usize, EcuError>;
}

#[embassy_executor::task()]
pub async fn server(
    mut sender: Sender<'static, NoopRawMutex, ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, ChannelItem>,

    #[cfg(any(feature = "elm327", feature = "l9637"))] adapter: crate::ecu_adapter::Adapter<
        'static,
    >,
) {
    // get a specific ECU
    #[cfg(feature = "ccm450")]
    let mut ecu = ECU::new(adapter);

    loop {
        // get a fresh channel item to write to
        let reply_item = sender.send().await;

        // waiting for request or reply
        match select(receiver.receive(), ecu.reply(&mut reply_item.data)).await {
            // received an ecu request
            Either::First(request_item) => {
                trace!(
                    "#### ECU: received: {}",
                    request_item.data[..request_item.size]
                );

                if let Some(request) = Request::from_repr(request_item.data[1] as usize) {
                    match request {
                        Request::Connect => {
                            if let Err(err) = ecu.connect().await {
                                error!("#### Request::Connect error: {}", err);
                                ecu_reply(reply_item, err);
                            } else {
                                trace!("#### Request::Connect Ok");
                                ecu_reply(reply_item, EcuError::Ok);
                            }
                            trace!("#### ECU: reply: {}", &reply_item.data[..reply_item.size]);
                            sender.send_done();
                        }
                        Request::Disconnect => {}
                        Request::LiveDataStart => {}
                        Request::LiveDataStop => {}
                    }
                    // we have finished to process request
                    receiver.receive_done();
                } else {
                    // send invalid request error via udp immediately
                    ecu_reply(reply_item, EcuError::InvalidRequest);
                    trace!("#### ECU: reply: {}", &reply_item.data[..reply_item.size]);
                    sender.send_done();
                }
            }

            // received reply from ecu
            Either::Second(result) => {
                crate::debug_pin::debug_pin(0);

                reply_item.size = result.unwrap_or_else(|_err| {
                    trace!("#### {}", _err);
                    0
                });

                // if not error forward reply to udp
                if reply_item.size > 0 {
                    trace!("#### ECU: reply: {}", &reply_item.data[..reply_item.size]);

                    // wake receiver
                    sender.send_done();
                }
            }
        }

        fn ecu_reply(reply_item: &mut ChannelItem, error: EcuError) {
            reply_item.data[0] = crate::udp::Subsystem::Ecu as u8;
            reply_item.size = 2;
            reply_item.data[1] = match error {
                EcuError::Ok => 0,
                EcuError::InvalidRequest => 1,
                EcuError::InconsystentRequest => 2,
                EcuError::CommunicationError(err) => match err {
                    AdapterError::Tx(_) => {
                        reply_item.data[2] = 0;
                        reply_item.size = 3;
                        3
                    }
                    AdapterError::Rx(err) => {
                        reply_item.data[2] = 1;
                        reply_item.data[3] = err as u8;
                        reply_item.size = 4;
                        3
                    }
                    AdapterError::Timeout => {
                        reply_item.data[2] = 3;
                        reply_item.size = 3;
                        3
                    }
                },
            } as u8;
        }
    }
}
