use embassy_futures::select::{Either, select};
use embassy_net::udp::{RecvError, SendError, UdpMetadata, UdpSocket};

// we can use NoopRawMutex since we use channel between two tasks in the same executor,
// in single core environment and not using from interrupt
//use esp_hal::sync::RawMutex;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    zerocopy_channel::{Receiver, Sender},
};

const UDP_PORT: Option<&'static str> = option_env!("UDP_PORT");

pub const UDP_BUFFER_SIZE: usize = crate::CHANNEL_ITEM_SIZE;
const UDP_PACKET_MAX: usize = 4;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Subsystem {
    // itself
    System,
    Ecu,
    Gps,
}

impl TryFrom<u8> for Subsystem {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::System),
            1 => Ok(Self::Ecu),
            2 => Ok(Self::Gps),
            _ => Err(()),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Response{
    #[allow(dead_code)]
    // RecvError: 0 = Truncated,
    UdpRecvError(RecvError),

    // SendError: 0 = NoRoute, 1 = SocketNotBound, 2 = PacketTooLarge,
    UdpSendError(SendError),

    InvalidSubsystem,
}

#[repr(u8)]
pub enum ResponseCode {
    UdpRecvError = 0x01,
    UdpSendError = 0x02,
    InvalidSubsystem = 0x03,
}

impl Response {
    pub fn code(&self) -> u8 {
        match self {
            Self::UdpRecvError(_) => ResponseCode::UdpRecvError as u8,
            Self::UdpSendError(_) => ResponseCode::UdpSendError as u8,
            Self::InvalidSubsystem => ResponseCode::InvalidSubsystem as u8,
        }
    }

    pub fn sub_code(&self) -> u8 {
        match self {
            // RecvError: 0 = Truncated,
            Self::UdpRecvError(err) => *err as u8,
            // SendError: 0 = NoRoute, 1 = SocketNotBound, 2 = PacketTooLarge,
            Self::UdpSendError(err) => *err as u8,
            Self::InvalidSubsystem => ResponseCode::InvalidSubsystem as u8,
        }
    }
}

#[embassy_executor::task()]
pub async fn server(
    ap_stack: embassy_net::Stack<'static>,
    mut ecu_sender: Sender<'static, NoopRawMutex, crate::ChannelItem>,
    mut ecu_receiver: Receiver<'static, NoopRawMutex, crate::ChannelItem>,
) {
    // setup UDP server socket
    use embassy_net::udp::PacketMetadata;

    let mut rx_meta = [PacketMetadata::EMPTY; UDP_PACKET_MAX];
    let mut tx_meta = [PacketMetadata::EMPTY; UDP_PACKET_MAX];
    let mut rx_buffer = [0; UDP_BUFFER_SIZE * UDP_PACKET_MAX];
    let mut tx_buffer = [0; UDP_BUFFER_SIZE * UDP_PACKET_MAX];

    let mut socket = UdpSocket::new(
        ap_stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let port = unwrap!(
        UDP_PORT.unwrap_or("19924").parse::<u16>(),
        "failed to parse UDP socket"
    );
    unwrap!(socket.bind(port));
    let mut end_point: Option<UdpMetadata> = None;

    loop {
        // wait for the channel to clear
        let request = ecu_sender.send().await;

        // waiting for udp request or ecu response
        match select(
            socket.recv_from(&mut request.data),
            ecu_receiver.receive(),
        )
        .await
        {
            // UDP request arrived
            Either::First(result) => {
                match result {
                    Ok((n, ep)) => {
                        trace!(
                            "#### UDP: socket.recv_from(): {:x}",
                            request.data[..n]
                        );

                        // first byte is the id of requested subsystem
                        if let Ok(subsystem) = Subsystem::try_from(request.data[0] as u8) {
                            match subsystem {
                                Subsystem::System => {}
                                Subsystem::Ecu => {
                                    // send all icoming data
                                    request.size = n;

                                    // send/forward request to ecu
                                    ecu_sender.send_done();
                                }
                                Subsystem::Gps => {}
                            }
                        } else {
                            // response error by System: InvalidSubystem
                            send_to(
                                &mut socket,
                                &[Subsystem::System as u8, Response::InvalidSubsystem.code()],
                                Some(ep)
                                )
                                .await;

                            debug!("#### UDP: send_to() returned");
                        }

                        end_point = Some(ep);
                    },

                    Err(recv_err) => {
                        let err_buf = [Subsystem::System as u8, ResponseCode::UdpRecvError as u8, recv_err as u8];
                        send_to(&mut socket, &err_buf, end_point).await;
                    }
                }
            }

            // Uart answer arrived
            Either::Second(response) => {
                trace!(
                    "#### UDP: ecu_receiver.receive(): size: {}",
                    response.size
                );

                // forward response via UDP
                send_to(&mut socket, &response.data[..response.size], end_point).await;

                debug!("#### UDP: send_to() returned");

                ecu_receiver.receive_done();
            }
        };

    }
}

// Calling a subfunction is almost always better than inlining the code into every error branch.
// Since error handling usually happens in multiple places but shouldn't trigger often, you
// want to optimize for Flash (code space) and Readability rather than raw execution speed.
//
// RAM
// The compiler treats these calls as a Union, the Task's storage struct only reserves one slot.
// The RAM footprint of the task does not increase beyond the size needed for a single call.
//
// Flash
// This function exists once in the binary.
// Each call is just a small bit of "glue" code to poll it.
#[allow(unused_must_use)]
async fn send_to(socket: &mut UdpSocket<'_>, buf: &[u8], end_point: Option<UdpMetadata>) {
    // unwrapping end_point cannot panics
    // UDP response cannot occur w/o a matching request
    // so end_point as Option used instead of static mut
    let end_point = unwrap!(end_point, "missing Endpoint");

    if let Err(err) = socket.send_to(buf, end_point)
        .await
        .map_err(|send_err|
            match send_err {
                SendError::PacketTooLarge => Response::UdpSendError(SendError::PacketTooLarge),
                _ => panic!("UDP send error: {:?}", send_err),
            })
        {
            let err_buf = [Subsystem::System as u8, err.code(), err.sub_code()];
            socket.send_to(&err_buf, end_point).await;
        }
}
