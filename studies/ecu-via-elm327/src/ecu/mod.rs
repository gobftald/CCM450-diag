#[cfg_attr(feature = "ccm450", path = "ccm450.rs")]
mod ecu_impl;
use ecu_impl::ECU;

// we can use NoopRawMutex since we use channel between two tasks in the
// same executor in single core environment and not using from interrupt
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};
use embassy_futures::select::{Either, select};

use crate::{ChannelItem, adapter::AdapterError};

#[allow(clippy::enum_variant_names)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum Request {
    Connect,
    ReadData,
    ReadDTC,
    ClearDTC,
    RawRequest,
    LiveDataStart,
    LiveDataStop,
}

impl TryFrom<u8> for Request {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Connect),
            1 => Ok(Self::ReadData),
            2 => Ok(Self::ReadDTC),
            3 => Ok(Self::ClearDTC),
            4 => Ok(Self::RawRequest),
            5 => Ok(Self::LiveDataStart),
            6 => Ok(Self::LiveDataStop),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum EcuError {
    Ok,
    InvalidRequest,
    NotConnected,
    AdapterError(AdapterError),
}

trait Ecu {
    async fn connect(&mut self) -> Result<(), EcuError>;
    async fn read_data(&mut self, ids: &[u8], reply: &mut [u8]) -> Result<usize, EcuError>;
    async fn read_dtc(&mut self, reply: &mut [u8]) -> Result<usize, EcuError>;
    async fn clear_dtc(&mut self) -> Result<(), EcuError>;
    async fn raw_request(&mut self, request: &[u8], reply: &mut [u8]) -> Result<usize, EcuError>;
    async fn reply(&mut self, reply: &mut [u8]) -> Result<usize, EcuError>;
}

#[embassy_executor::task()]
pub async fn server(
    adapter: crate::adapter::Adapter<'static>,
    mut sender: Sender<'static, NoopRawMutex, ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, ChannelItem>,
) {
    // get a specific ECU
    #[cfg(feature = "ccm450")]
    let mut ecu = ECU::new(adapter);

    loop {
        // get a fresh channel item to write to
        let reply = sender.send().await;

        // waiting for request or reply
        match select(receiver.receive(), ecu.reply(&mut reply.data)).await {
            // received an ecu request
            Either::First(request) => {
                trace!("#### ECU: received: {}", request.data[..request.size]);

                if let Ok(reqst) = Request::try_from(request.data[1] as u8) {
                    match reqst {
                        Request::Connect => {
                            if let Err(err) = ecu.connect().await {
                                ecu_reply(reply, err);
                            } else {
                                ecu_reply(reply, EcuError::Ok);
                            }
                            sender.send_done();
                        }

                        Request::ReadData => {
                            // check that input(s) are 16-bit common identifier(s)
                            if request.data.len() % 2 != 0 {
                                ecu_reply(reply, EcuError::InvalidRequest);
                            }

                            match ecu
                                .read_data(
                                    // common identifier(s)
                                    &request.data[2..request.size],
                                    // borrow the full buffer
                                    &mut reply.data[..(crate::CHANNEL_ITEM_SIZE)],
                                )
                                .await
                            {
                                Ok(size) => {
                                    // copy the payload of ecu answer
                                    ecu_reply(reply, EcuError::Ok);
                                    // setting the size to sending the payload of got reply
                                    reply.size = size;
                                }
                                Err(err) => {
                                    ecu_reply(reply, err);
                                }
                            }
                            sender.send_done();
                        }
                        Request::ReadDTC => {
                            match ecu
                                .read_dtc(&mut reply.data[..(crate::CHANNEL_ITEM_SIZE)])
                                .await
                            {
                                Ok(size) => {
                                    // copy the payload of ecu answer
                                    ecu_reply(reply, EcuError::Ok);
                                    // setting the size to sending the payload of got reply
                                    reply.size = size;
                                }
                                Err(err) => {
                                    ecu_reply(reply, err);
                                }
                            }
                            sender.send_done();
                        }
                        Request::ClearDTC => {
                            if let Err(err) = ecu.clear_dtc().await {
                                ecu_reply(reply, err);
                            } else {
                                ecu_reply(reply, EcuError::Ok);
                            }
                            sender.send_done();
                        }
                        Request::RawRequest => {
                            match ecu
                                .raw_request(
                                    &request.data[2..request.size],
                                    &mut reply.data[..(crate::CHANNEL_ITEM_SIZE)],
                                )
                                .await
                            {
                                Ok(size) => {
                                    // copy the payload of ecu answer
                                    ecu_reply(reply, EcuError::Ok);
                                    // setting the size to sending the payload of got reply
                                    reply.size = size;
                                }
                                Err(err) => {
                                    ecu_reply(reply, err);
                                }
                            }
                            sender.send_done();
                        }
                        Request::LiveDataStart => {}
                        Request::LiveDataStop => {}
                    }
                    // we have finished to process request
                    receiver.receive_done();
                } else {
                    // send invalid request error via udp immediately
                    ecu_reply(reply, EcuError::InvalidRequest);
                    trace!("#### ECU: reply: {:a}", &reply.data[..reply.size]);
                    sender.send_done();
                }
            }

            // received reply from ecu
            Either::Second(result) => {
                reply.size = result.unwrap_or_else(|_err| {
                    trace!("#### ECU #direct/unwaited# error: {}", _err);
                    0
                });

                // if not error forward reply to udp
                if reply.size > 0 {
                    trace!(
                        "#### ECU #direct/unwaited# reply: {:a}",
                        &reply.data[..reply.size]
                    );

                    // wake receiver
                    sender.send_done();
                }
            }
        }

        // complete the reply frame
        fn ecu_reply(reply: &mut ChannelItem, error: EcuError) {
            reply.data[0] = crate::udp::Subsystem::Ecu as u8;
            reply.size = 2;
            reply.data[1] = match error {
                EcuError::Ok => 0,
                EcuError::InvalidRequest => 1,
                EcuError::NotConnected => 2,
                EcuError::AdapterError(error) => match error {
                    AdapterError::Rx(err) => {
                        reply.data[2] = error.into();
                        reply.data[3] = err as u8;
                        reply.size = 4;
                        3
                    }
                    AdapterError::EcuSpecificError(err) => {
                        reply.data[2] = error.into();
                        reply.data[3] = err;
                        reply.size = 4;
                        3
                    }
                    _ => {
                        reply.data[2] = error.into();
                        reply.size = 3;
                        3
                    }
                },
            } as u8;
        }
    }
}
