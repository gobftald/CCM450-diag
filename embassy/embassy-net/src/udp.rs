//! UDP sockets.

// 3
use core::future::{poll_fn, Future};
use core::mem;
use core::task::{Context, Poll};

// 7
use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::udp;
pub use smoltcp::socket::udp::{PacketMetadata, UdpMetadata};
use smoltcp::wire::IpListenEndpoint;

// 12
use crate::Stack;

/// Error returned by [`UdpSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 17
pub enum BindError {
    /// The socket was already open.
    InvalidState,
    /// No route to host.
    NoRoute,
}

/// Error returned by [`UdpSocket::recv_from`] and [`UdpSocket::send_to`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 27
pub enum SendError {
    /// No route to host.
    NoRoute,
    /// Socket not bound to an outgoing port.
    SocketNotBound,
}

/// Error returned by [`UdpSocket::recv_from`] and [`UdpSocket::send_to`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 37
pub enum RecvError {
    /// Provided buffer was smaller than the received packet.
    Truncated,
}

/// An UDP socket.
// 43
pub struct UdpSocket<'a> {
    stack: Stack<'a>,
    handle: SocketHandle,
}

// 48
impl<'a> UdpSocket<'a> {
    /// Create a new UDP socket using the provided stack and buffers.
    // 50
    pub fn new(
        stack: Stack<'a>,
        rx_meta: &'a mut [PacketMetadata],
        rx_buffer: &'a mut [u8],
        tx_meta: &'a mut [PacketMetadata],
        tx_buffer: &'a mut [u8],
    ) -> Self {
        let handle = stack.with_mut(|i| {
            let rx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(rx_meta) };
            let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
            let tx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(tx_meta) };
            let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
            i.sockets.add(udp::Socket::new(
                udp::PacketBuffer::new(rx_meta, rx_buffer),
                udp::PacketBuffer::new(tx_meta, tx_buffer),
            ))
        });

        Self { stack, handle }
    }

    /// Bind the socket to a local endpoint.
    // 72
    pub fn bind<T>(&mut self, endpoint: T) -> Result<(), BindError>
    where
        T: Into<IpListenEndpoint>,
    {
        let mut endpoint = endpoint.into();

        if endpoint.port == 0 {
            // If user didn't specify port allocate a dynamic port.
            endpoint.port = self.stack.with_mut(|i| i.get_local_port());
        }

        match self.with_mut(|s, _| s.bind(endpoint)) {
            Ok(()) => Ok(()),
            Err(udp::BindError::InvalidState) => Err(BindError::InvalidState),
            Err(udp::BindError::Unaddressable) => Err(BindError::NoRoute),
        }
    }

    // 97
    fn with_mut<R>(&self, f: impl FnOnce(&mut udp::Socket, &mut Interface) -> R) -> R {
        self.stack.with_mut(|i| {
            let socket = i.sockets.get_mut::<udp::Socket>(self.handle);
            let res = f(socket, &mut i.iface);
            // triggering poll
            i.waker.wake();
            res
        })
    }

    /// Receive a datagram.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// Returns the number of bytes received and the remote endpoint.
    // 137
    pub fn recv_from<'s>(
        &'s self,
        buf: &'s mut [u8],
    ) -> impl Future<Output = Result<(usize, UdpMetadata), RecvError>> + 's {
        poll_fn(|cx| self.poll_recv_from(buf, cx))
    }

    /// Receive a datagram.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready` with the
    /// number of bytes received and the remote endpoint.
    // 151
    pub fn poll_recv_from(
        &self,
        buf: &mut [u8],
        cx: &mut Context<'_>,
    ) -> Poll<Result<(usize, UdpMetadata), RecvError>> {
        self.with_mut(|s, _| match s.recv_slice(buf) {
            Ok((n, meta)) => Poll::Ready(Ok((n, meta))),
            // No data ready
            Err(udp::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            Err(udp::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// This method will wait until the datagram has been sent.
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(SendError::NoRoute)`
    // 228
    pub async fn send_to<T>(&self, buf: &[u8], remote_endpoint: T) -> Result<(), SendError>
    where
        T: Into<UdpMetadata>,
    {
        let remote_endpoint: UdpMetadata = remote_endpoint.into();
        poll_fn(move |cx| self.poll_send_to(buf, remote_endpoint, cx)).await
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the socket's send buffer is full, this method will return `Poll::Pending`
    /// and register the current task to be notified when the buffer has space available.
    ///
    /// When the remote endpoint is not reachable, this method will return `Poll::Ready(Err(Error::NoRoute))`.
    // 244
    pub fn poll_send_to<T>(
        &self,
        buf: &[u8],
        remote_endpoint: T,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SendError>>
    where
        T: Into<UdpMetadata>,
    {
        self.with_mut(|s, _| match s.send_slice(buf, remote_endpoint) {
            // Entire datagram has been sent
            Ok(()) => Poll::Ready(Ok(())),
            Err(udp::SendError::BufferFull) => {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
            Err(udp::SendError::Unaddressable) => {
                // If no sender/outgoing port is specified, there is not really "no route"
                if s.endpoint().port == 0 {
                    Poll::Ready(Err(SendError::SocketNotBound))
                } else {
                    Poll::Ready(Err(SendError::NoRoute))
                }
            }
        })
    }
}

// 368
impl Drop for UdpSocket<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.sockets.remove(self.handle));
    }
}
