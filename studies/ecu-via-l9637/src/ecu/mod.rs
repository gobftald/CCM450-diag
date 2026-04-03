#[cfg_attr(feature = "ccm450", path = "ccm450.rs")]
mod ecu_impl;
use ecu_impl::ECU;

// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
// the more future-proof ThreadModeRawMutex is implemented only for cortex_m in embassy_sync
// and CriticalSectionRawMutex is unecessary for this case
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};
use embassy_futures::select::{Either, select};

use crate::ChannelItem;
// adapter and ecu specific errors
use crate::{adapter::AdapterError, protocol::EcuError};

macro_rules! ecu_api {
    ($($enum_variant:ident $enum_id:literal $fn_name:ident ( $($arg_name:ident : $arg_type:ty),* ) $(-> $ret:ty)? );* $(;)?) => {
        
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum EcuRequest {
            $($enum_variant),*
        }

        impl TryFrom<u8> for EcuRequest {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $( $enum_id => Ok(Self::$enum_variant), )*
                    _ => Err(()),
                }
            }
        }

        pub trait EcuApi {
            $(
                async fn $fn_name(&mut self, $($arg_name : $arg_type),*) $(-> $ret)?;
            )*

            async fn reply(&mut self, reply: &mut [u8]) -> Result<usize, Error>;
            }
    }
}

// Map EcuRequest enum and EcuApi trait together tightly
ecu_api! {
    Connect    0  connect() -> Result<(), Error>;
    ReadData   1  read_data(ids: &[u8], reply: &mut [u8]) -> Result<usize, Error>;
    ReadDTC    2  read_dtc(reply: &mut [u8]) -> Result<usize, Error>;
    ClearDTC   3  clear_dtc() -> Result<(), Error>;
    RawRequest 4  raw_request(request: &[u8], reply: &mut [u8]) -> Result<usize, Error>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EcuApiError {
    InvalidRequest,
    NotConnected,
    AlreadyConnected,
}

// aggregate error type for response
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Ok,
    EcuApiError(EcuApiError),
    AdapterError(AdapterError),
    EcuError(EcuError),
}

impl From<EcuApiError> for Error {
    fn from(err: EcuApiError) -> Self {
        Error::EcuApiError(err)
    }
}

impl From<AdapterError> for Error {
    fn from(err: AdapterError) -> Self {
        Error::AdapterError(err)
    }
}

#[embassy_executor::task()]
pub async fn server(
    adapter: crate::adapter::Adapter<'static>,
    mut sender: Sender<'static, NoopRawMutex, ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, ChannelItem>,
) {
    // Get ECU
    //
    // ECU and Protocol specification handled by feature gated modules.
    // see e.g. ecu_impl::ECU above
    // Specific Adapter as the only parameter comes from the main module,
    // because specific Periherals can be created only there.
    let mut ecu = ECU::new(adapter);

    loop {
        // get a fresh channel item to write to
        let reply = sender.send().await;

        // waiting for request or reply
        match select(receiver.receive(), ecu.reply(&mut reply.data)).await {
            // received an ecu request
            Either::First(request) => {
                trace!("#### ECU: received: {}", request.data[..request.size]);

                if let Ok(reqst) = EcuRequest::try_from(request.data[1] as u8) {
                    match reqst {
                        EcuRequest::Connect => {
                            if let Err(err) = ecu.connect().await {
                                ecu_reply(reply, err);
                            } else {
                                ecu_reply(reply, Error::Ok);
                            }
                            sender.send_done();
                        }

                        EcuRequest::ReadData => {
                            // check that input(s) are 16-bit common identifier(s)
                            if request.data.len() % 2 != 0 {
                                ecu_reply(reply, EcuApiError::InvalidRequest.into());
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
                                    ecu_reply(reply, Error::Ok);
                                    // setting the size to sending the payload of got reply
                                    reply.size = size;
                                }
                                Err(err) => {
                                    ecu_reply(reply, err);
                                }
                            }
                            sender.send_done();
                        }
                        EcuRequest::ReadDTC => {
                            match ecu
                                .read_dtc(&mut reply.data[..(crate::CHANNEL_ITEM_SIZE)])
                                .await
                            {
                                Ok(size) => {
                                    // copy the payload of ecu answer
                                    ecu_reply(reply, Error::Ok);
                                    // setting the size to sending the payload of got reply
                                    reply.size = size;
                                }
                                Err(err) => {
                                    ecu_reply(reply, err);
                                }
                            }
                            sender.send_done();
                        }
                        EcuRequest::ClearDTC => {
                            if let Err(err) = ecu.clear_dtc().await {
                                ecu_reply(reply, err);
                            } else {
                                ecu_reply(reply, Error::Ok);
                            }
                            sender.send_done();
                        }
                        EcuRequest::RawRequest => {
                            match ecu
                                .raw_request(
                                    &request.data[2..request.size],
                                    &mut reply.data[..(crate::CHANNEL_ITEM_SIZE)],
                                )
                                .await
                            {
                                Ok(size) => {
                                    // copy the payload of ecu answer
                                    ecu_reply(reply, Error::Ok);
                                    // setting the size to sending the payload of got reply
                                    reply.size = size;
                                }
                                Err(err) => {
                                    ecu_reply(reply, err);
                                }
                            }
                            sender.send_done();
                        }
                    }
                    // we have finished to process request
                    receiver.receive_done();
                } else {
                    // send invalid request error via udp immediately
                    ecu_reply(reply, EcuApiError::InvalidRequest.into());
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
        fn ecu_reply(reply: &mut ChannelItem, error: Error) {
            reply.data[0] = crate::udp::Subsystem::Ecu as u8;
            reply.size = 2;
            reply.data[1] = match error {
                Error::Ok => 0,
                Error::EcuApiError(error) => match error {
                    EcuApiError::InvalidRequest => 1,
                    EcuApiError::NotConnected => 2,
                    EcuApiError::AlreadyConnected => 3,
                },
                Error::AdapterError(error) => match error {
                    AdapterError::RxError(err) => {
                        reply.data[2] = error.into();
                        reply.data[3] = err as u8;
                        reply.size = 4;
                        4
                    },
                    AdapterError::TxError(err) => {
                        reply.data[2] = error.into();
                        reply.data[3] = err as u8;
                        reply.size = 4;
                        4
                    },
                },
                Error::EcuError(error) => {
                    reply.data[2] = error.0 as u8;
                    5
                },
            } as u8;
        }
    }
}
