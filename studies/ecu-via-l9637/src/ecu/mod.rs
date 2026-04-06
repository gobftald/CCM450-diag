#[cfg_attr(feature = "ccm450", path = "ccm450.rs")]
mod ecu_impl;
use ecu_impl::ECU;

pub use ecu_impl::EcuError;

// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
// the more future-proof ThreadModeRawMutex is implemented only for cortex_m in embassy_sync
// and CriticalSectionRawMutex is unecessary for this case
use embassy_sync::{
    blocking_mutex::raw::{NoopRawMutex, RawMutex},
    zerocopy_channel::{Receiver, Sender},
    signal::Signal,
};

use embassy_futures::select::{Either3, select3};

use crate::ChannelItem;
// adapter and ecu specific errors
use crate::{adapter::AdapterError, protocol::ProtocolError};

// force Sync for NoopRawMutex to make a static Signal
// it is safe since signal will not be used in interrupt,
// but only in "ThreadMode" in the same Executor
pub struct SyncNoopRawMutex(NoopRawMutex);

unsafe impl Sync for SyncNoopRawMutex {}

// We must also implement RawMutex by forwarding to the inner Noop
unsafe impl RawMutex for SyncNoopRawMutex {
    const INIT: Self = Self(NoopRawMutex::INIT);
    fn lock<R>(&self, f: impl FnOnce() -> R) -> R { self.0.lock(f) }
}

static TESTER_PRESENT: Signal<SyncNoopRawMutex, bool> = Signal::new();

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

            async fn tester_present(&mut self);
            async fn response(&mut self, response: &mut [u8]) -> Result<usize, Error>;
            }
    }
}

// Map EcuRequest enum and EcuApi trait together tightly
ecu_api! {
    Connect    0  connect() -> Result<usize, Error>;
    RawRequest 1  raw_request(request: &[u8], response: &mut [u8]) -> Result<usize, Error>;
    ReadDTC    2  read_dtc(response: &mut [u8]) -> Result<usize, Error>;
    ClearDTC   3  clear_dtc() -> Result<(), Error>;
    ReadData   4  read_data(ids: &[u8], response: &mut [u8]) -> Result<usize, Error>;
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
    ProtocolError(ProtocolError),
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

impl From<ProtocolError> for Error {
    fn from(err: ProtocolError) -> Self {
        Error::ProtocolError(err)
    }
}

#[embassy_executor::task()]
pub async fn server(
    spawner: embassy_executor::Spawner,
    adapter: crate::adapter::Adapter<'static>,
    mut sender: Sender<'static, NoopRawMutex, ChannelItem>,
    mut receiver: Receiver<'static, NoopRawMutex, ChannelItem>,
) {
    // Get ECU
    //
    // ECU and Protocol specification handled by feature gated modules.
    // see e.g. ecu_impl::ECU above
    // Specific Adapter as the only parameter comes from the main module,
    // because specific Periherals can be created and used only there.
    let mut ecu = ECU::new(adapter);

    loop {
        // get a fresh channel item to write to
        let response = sender.send().await;

        // waiting for request or response
        match select3(
            receiver.receive(),
            ecu.response(&mut response.data),
            TESTER_PRESENT.wait()
        ).await {
            // received an ecu request
            Either3::First(request) => {
                trace!("#### ECU: received: {}", request.data[..request.size]);

                if let Ok(reqst) = EcuRequest::try_from(request.data[1] as u8) {
                    match reqst {
                        EcuRequest::Connect => {
                            response.size = ecu_response(response, ecu.connect().await);
                            spawner.spawn(tester_present()).ok();
                            sender.send_done();
                        }

                        EcuRequest::RawRequest => {
                            if request.size > 2 {
                                let result = ecu.raw_request(
                                    &request.data[2..request.size],
                                    &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                                )
                                .await;
                                response.size = ecu_response(response, result) ;
                                sender.send_done();
                            } else {
                                // send invalid request error via udp immediately
                                response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                                trace!("#### ECU: response: {:a}", &response.data[..response.size]);
                                sender.send_done();
                            }
                        }

                        EcuRequest::ReadDTC => {
                            let result = ecu.read_dtc(
                                &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                            )
                            .await;
                            response.size = ecu_response(response, result) ;
                            sender.send_done();
                        }

                        EcuRequest::ClearDTC => {
                            if let Err(err) = ecu.clear_dtc().await {
                                ecu_response(response, Err(err));
                            } else {
                                ecu_response(response, Err(Error::Ok));
                            }
                            sender.send_done();
                        }

                        EcuRequest::ReadData => {
                            // check that input(s) are 16-bit common identifier(s)
                            if request.data.len() % 2 != 0 {
                                ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                            }

                            match ecu
                                .read_data(
                                    // common identifier(s)
                                    &request.data[2..request.size],
                                    // borrow the full buffer
                                    &mut response.data[..(crate::CHANNEL_ITEM_SIZE)],
                                )
                                .await
                            {
                                Ok(size) => {
                                    // copy the payload of ecu answer
                                    ecu_response(response, Err(Error::Ok));
                                    // setting the size to sending the payload of got response
                                    response.size = size;
                                }
                                Err(err) => {
                                    ecu_response(response, Err(err));
                                }
                            }
                            sender.send_done();
                        }

                    }
                    // we have finished to process request
                    receiver.receive_done();
                } else {
                    // send invalid request error via udp immediately
                    response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                    trace!("#### ECU: response: {:a}", &response.data[..response.size]);
                    sender.send_done();
                }
            }

            // received response from ecu
            Either3::Second(result) => {
                response.size = result.unwrap_or_else(|_err| {
                    trace!("#### ECU #direct/unwaited# error: {}", _err);
                    0
                });

                // if not error forward response to udp
                if response.size > 0 {
                    trace!(
                        "#### ECU #direct/unwaited# response: {:a}",
                        &response.data[..response.size]
                    );

                    // wake receiver
                    sender.send_done();
                }
            }

            // received TESTER_RESENT signal
            Either3::Third(_signal) => {
                ecu.tester_present().await;
                TESTER_PRESENT.reset();
            }
        }

        // complete the response frame
        fn ecu_response(response: &mut ChannelItem, result: Result<usize, Error>) -> usize {
            let mut rsize1 = 0;
            let mut rsize2 = 0;

            response.data[0] = crate::udp::Subsystem::Ecu as u8;
            response.data[1] = result.map_or_else(|error| match error {
                // Not used this way, it is only a placeholder
                Error::Ok => 0,
                Error::EcuApiError(error) => {
                    response.data[2] = error as u8;
                    rsize1 = 3;
                    1
                },
                Error::AdapterError(error) => match error {
                    AdapterError::RxError(err) => {
                        response.data[2] = error.into();
                        response.data[3] = err as u8;
                        rsize1 = 4;
                        2
                    },
                    AdapterError::TxError(err) => {
                        response.data[2] = error.into();
                        response.data[3] = err as u8;
                        rsize1 = 4;
                        2
                    },
                },
                Error::ProtocolError(error) => match error{
                    ProtocolError::InvalidChecksum => {
                        response.data[2] = error.into();
                        rsize1 = 3;
                        3
                    },
                    ProtocolError::EcuError(err) => {
                        response.data[2] = error.into();
                        response.data[3] = err.0;
                        rsize1 = 4;
                        3
                    },
                }
            // if Ok, size information is not set here
            // Error::Ok => 0
            }, |size| {rsize2 = size + 2; 0});

        rsize1 + rsize2
        }
    }
}

#[embassy_executor::task()]
async fn tester_present() {
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(4_500)).await;
        TESTER_PRESENT.signal(true);
    }
}
