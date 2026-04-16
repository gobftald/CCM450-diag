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
    ($($variant:ident $id:literal $kind:ident $fn:ident ( $($arg:ident : $type:ty),* ) $(-> $ret:ty)? );* $(;)?) => {
        
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum EcuRequest {
            $($variant),*
        }

        impl TryFrom<u8> for EcuRequest {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $( $id => Ok(Self::$variant), )*
                    _ => Err(()),
                }
            }
        }

        pub trait EcuApi {
            $(
                ecu_api!(@expand_fn $kind $fn ( $($arg : $type),* ) $(-> $ret)?);
            )*

            async fn tester_present(&mut self);
            
            // not used yet
            async fn response(&mut self, response: &mut [u8]) -> Result<usize, Error>;
        }
    };

    (@expand_fn async $fn:ident ($($args:tt)*) $(-> $ret:ty)?) => {
        async fn $fn(&mut self, $($args)*) $(-> $ret)?;
    };

    (@expand_fn sync $fn:ident ($($args:tt)*) $(-> $ret:ty)?) => {
        fn $fn(&mut self, $($args)*) $(-> $ret)?;
    };
}

// Map EcuRequest enum and EcuApi trait together tightly
ecu_api! {
    Connect         0   async connect() -> Result<usize, Error>;
    RawRequest      1   async raw_request(request: &[u8], response: &mut [u8]) -> Result<usize, Error>;

    ReadDTC         2   async read_dtc(response: &mut [u8]) -> Result<usize, Error>;
    ClearDTC        3   async clear_dtc() -> Result<usize, Error>;

    SecurityAccess  4   async security_access() -> Result<usize, Error>;

    ReadDataIds     5   sync  read_data_ids(response: &mut [u8]) -> usize;
    DataIdDescr     6   sync  data_id_description(id: &[u8], response: &mut [u8]) -> Result<usize, Error>;
    ReadData        7   async read_data(id: &[u8], response: &mut [u8]) -> Result<usize, Error>;

    ReadSartIds     8   sync  read_start_ids(response: &mut [u8]) -> usize;
    ReadStopIds     9   sync  read_stop_ids(response: &mut [u8]) -> usize;
    StartIdDescr   10   sync  start_id_description(id: &[u8], response: &mut [u8]) -> Result<usize, Error>;
    StopIdDescr    11   sync  stop_id_description(id: &[u8], response: &mut [u8]) -> Result<usize, Error>;
    StartRoutine   12   async start_routine(arg: &[u8], response: &mut [u8]) -> Result<usize, Error>;
    StopRoutine    13   async stop_routine(arg: &[u8], response: &mut [u8]) -> Result<usize, Error>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EcuApiError {
    InvalidRequest,
    NotConnected,
    AlreadyConnected,
    InvalidId,
}

// aggregate error type for response
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(dead_code)]
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
        let mut response = sender.send().await;

        // waiting for request or response
        match select3(
            receiver.receive(),
            ecu.response(&mut response.data),
            TESTER_PRESENT.wait()
        ).await {
            // received an ecu request
            Either3::First(request) => {
                // for one request with multiple packet response
                let mut skip_receive_done = false;
                trace!("#### ECU: channel receiver.receive(): {:x}", request.data[..request.size]);

                if let Ok(reqst) = EcuRequest::try_from(request.data[1] as u8) {
                    match reqst {
                        EcuRequest::Connect => {
                            response.size = ecu_response(response, ecu.connect().await);
                            spawner.spawn(tester_present()).ok();
                        }

                        EcuRequest::RawRequest => {
                            if request.size > 2 {
                                let result = ecu.raw_request(
                                    &request.data[2..request.size],
                                    &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                                )
                                .await;
                                response.size = ecu_response(response, result) ;
                            } else {
                                // send invalid request error via udp immediately
                                response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                                trace!("#### ECU: response: {:a}", &response.data[..response.size]);
                            }
                        }

                        EcuRequest::ReadDTC => {
                            let result = ecu.read_dtc(
                                &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                            )
                            .await;
                            response.size = ecu_response(response, result) ;
                        }

                        EcuRequest::ClearDTC => {
                            response.size = ecu_response(response, ecu.clear_dtc().await);
                        }

                        EcuRequest::SecurityAccess => {
                            response.size = ecu_response(response, ecu.security_access().await);
                        }

                        EcuRequest::ReadDataIds => {
                            loop {
                                let size = ecu.read_data_ids(
                                    &mut response.data[2..]
                                );
                                response.size = ecu_response(response, Ok(size))    ;

                                if response.data[4] == 0 && response.data[5] == 0 {
                                    break;
                                } else {
                                    if !skip_receive_done {
                                        receiver.receive_done();
                                        // skip receive done for all subsequent packets
                                        skip_receive_done = true;
                                    }
                                    sender.send_done();

                                    response = sender.send().await;
                                }
                            }
                        }

                        EcuRequest::DataIdDescr => {
                            if request.size == 4 {
                                let result = ecu.data_id_description(
                                    &request.data[2..request.size],
                                    &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                                );
                                response.size = ecu_response(response, result) ;
                            } else {
                                // send invalid request error via udp immediately
                                response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                            }
                        }

                        EcuRequest::ReadData => {
                            if request.size == 4 {
                                let result = ecu.read_data(
                                    &request.data[2..request.size],
                                    &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                                )
                                .await;
                                response.size = ecu_response(response, result) ;
                            } else {
                                // send invalid request error via udp immediately
                                response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                            }
                        }

                        EcuRequest::ReadSartIds => {
                            let size = ecu.read_start_ids(
                                    &mut response.data[2..]
                                );
                            response.size = ecu_response(response, Ok(size))    ;
                        }

                        EcuRequest::ReadStopIds => {
                            let size = ecu.read_stop_ids(
                                    &mut response.data[2..]
                                );
                            response.size = ecu_response(response, Ok(size))    ;
                        }

                        EcuRequest::StartIdDescr => {
                            if request.size == 4 {
                                let result = ecu.start_id_description(
                                    &request.data[2..request.size],
                                    &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                                );
                                response.size = ecu_response(response, result) ;
                            } else {
                                // send invalid request error via udp immediately
                                response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                            }
                        }

                        EcuRequest::StopIdDescr => {
                            if request.size == 4 {
                                let result = ecu.stop_id_description(
                                    &request.data[2..request.size],
                                    &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                                );
                                response.size = ecu_response(response, result) ;
                            } else {
                                // send invalid request error via udp immediately
                                response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                            }
                        }

                        EcuRequest::StartRoutine => {
                            let result = ecu.start_routine(
                                &request.data[2..request.size],
                                &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                            )
                            .await;
                            response.size = ecu_response(response, result) ;
                        }

                        EcuRequest::StopRoutine => {
                            let result = ecu.stop_routine(
                                &request.data[2..request.size],
                                &mut response.data[2..(crate::CHANNEL_ITEM_SIZE)]
                            )
                            .await;
                            response.size = ecu_response(response, result) ;
                        }
                    }
                } else {
                    // send invalid request error via udp immediately
                    response.size = ecu_response(response, Err(EcuApiError::InvalidRequest.into()));
                }
                // we have finished to process request
                //
                // but we should not send receive_done signal
                // if we send multiple messages for a request (ReadIds)
                if !skip_receive_done {
                    receiver.receive_done();
                }

                // signal sent message packet
                sender.send_done();
            }

            // received response from ecu
            Either3::Second(result) => {
                response.size = result.unwrap_or_else(|_err| {
                    trace!("#### ECU #direct/unwaited# error: {:x}", _err);
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
                Error::Ok => {
                    rsize1 = 2;
                    0
                },
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
                    ProtocolError::Timeout => {
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
