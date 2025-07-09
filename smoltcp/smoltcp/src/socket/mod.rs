#[cfg(feature = "socket-dhcpv4")]
// 18
pub mod dhcpv4;

#[cfg(feature = "async")]
// 31
mod waker;

#[cfg(feature = "async")]
// 33
pub(crate) use self::waker::WakerRegistration;

/// A network socket.
///
/// This enumeration abstracts the various types of sockets based on the IP protocol.
/// To downcast a `Socket` value to a concrete socket, use the [AnySocket] trait,
/// e.g. to get `udp::Socket`, call `udp::Socket::downcast(socket)`.
///
/// It is usually more convenient to use [SocketSet::get] instead.
///
/// [AnySocket]: trait.AnySocket.html
/// [SocketSet::get]: struct.SocketSet.html#method.get
#[derive(Debug)]
// 59
pub enum Socket<'a> {
    //#[cfg(feature = "socket-raw")]
    //Raw(raw::Socket<'a>),
    //#[cfg(feature = "socket-icmp")]
    //Icmp(icmp::Socket<'a>),
    //#[cfg(feature = "socket-udp")]
    //Udp(udp::Socket<'a>),
    //#[cfg(feature = "socket-tcp")]
    //Tcp(tcp::Socket<'a>),
    #[cfg(feature = "socket-dhcpv4")]
    Dhcpv4(dhcpv4::Socket<'a>),
    //#[cfg(feature = "socket-dns")]
    //Dns(dns::Socket<'a>),
}

/// A conversion trait for network sockets.
// 94
pub trait AnySocket<'a> {
    fn upcast(self) -> Socket<'a>;
    fn downcast<'c>(socket: &'c Socket<'a>) -> Option<&'c Self>
    where
        Self: Sized;
    fn downcast_mut<'c>(socket: &'c mut Socket<'a>) -> Option<&'c mut Self>
    where
        Self: Sized;
}

// 104
macro_rules! from_socket {
    ($socket:ty, $variant:ident) => {
        impl<'a> AnySocket<'a> for $socket {
            fn upcast(self) -> Socket<'a> {
                Socket::$variant(self)
            }

            fn downcast<'c>(socket: &'c Socket<'a>) -> Option<&'c Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Socket::$variant(socket) => Some(socket),
                    _ => None,
                }
            }

            fn downcast_mut<'c>(socket: &'c mut Socket<'a>) -> Option<&'c mut Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Socket::$variant(socket) => Some(socket),
                    _ => None,
                }
            }
        }
    };
}

// 138
#[cfg(feature = "socket-dhcpv4")]
from_socket!(dhcpv4::Socket<'a>, Dhcpv4);
