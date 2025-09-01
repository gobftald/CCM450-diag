#[cfg(feature = "async")]
// 7
use crate::socket::WakerRegistration;
use crate::socket::{Context, PollAt};

// 10
use crate::storage::Empty;

#[cfg(feature = "proto-ipv4")]
// 13
use crate::wire::{Icmpv4Packet, Icmpv4Repr};

// 16
use crate::wire::{IpAddress, IpListenEndpoint};

/// Type of endpoint to bind the ICMP socket to. See [IcmpSocket::bind] for
/// more details.
///
/// [IcmpSocket::bind]: struct.IcmpSocket.html#method.bind
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 85
pub enum Endpoint {
    #[default]
    Unspecified,
    Ident(u16),
    Udp(IpListenEndpoint),
}

/// An ICMP packet ring buffer.
// 106
pub type PacketBuffer<'a> = crate::storage::PacketBuffer<'a, IpAddress>;

/// A ICMP socket
///
/// An ICMP socket is bound to a specific [IcmpEndpoint] which may
/// be a specific UDP port to listen for ICMP error messages related
/// to the port or a specific ICMP identifier value. See [bind] for
/// more details.
///
/// [IcmpEndpoint]: enum.IcmpEndpoint.html
/// [bind]: #method.bind
#[derive(Debug)]
// 118
pub struct Socket<'a> {
    rx_buffer: PacketBuffer<'a>,
    tx_buffer: PacketBuffer<'a>,
    /// The endpoint this socket is communicating with
    endpoint: Endpoint,
    /// The time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    hop_limit: Option<u8>,
    #[cfg(feature = "async")]
    rx_waker: WakerRegistration,
    #[cfg(feature = "async")]
    tx_waker: WakerRegistration,
}

// 131
impl<'a> Socket<'a> {
    /// Fitler determining whether the socket accepts a given ICMPv4 packet.
    /// Accepted packets are enqueued into the socket's receive buffer.
    #[cfg(feature = "proto-ipv4")]
    #[inline]
    // 429
    pub(crate) fn accepts_v4(
        &self,
        cx: &mut Context,
        ip_repr: &Ipv4Repr,
        icmp_repr: &Icmpv4Repr,
    ) -> bool {
        match (&self.endpoint, icmp_repr) {
            // If we are bound to ICMP errors associated to a UDP port, only
            // accept Destination Unreachable or Time Exceeded messages with
            // the data containing a UDP packet send from the local port we
            // are bound to.
            (
                &Endpoint::Udp(endpoint),
                &Icmpv4Repr::DstUnreachable { data, header, .. }
                | &Icmpv4Repr::TimeExceeded { data, header, .. },
            ) if endpoint.addr.is_none() || endpoint.addr == Some(ip_repr.dst_addr.into()) => {
                let packet = UdpPacket::new_unchecked(data);
                match UdpRepr::parse(
                    &packet,
                    &header.src_addr.into(),
                    &header.dst_addr.into(),
                    &cx.checksum_caps(),
                ) {
                    Ok(repr) => endpoint.port == repr.src_port,
                    Err(_) => false,
                }
            }
            // If we are bound to a specific ICMP identifier value, only accept an
            // Echo Request/Reply with the identifier field matching the endpoint
            // port.
            (&Endpoint::Ident(bound_ident), &Icmpv4Repr::EchoRequest { ident, .. })
            | (&Endpoint::Ident(bound_ident), &Icmpv4Repr::EchoReply { ident, .. }) => {
                ident == bound_ident
            }
            _ => false,
        }
    }

    // 561
    pub(crate) fn dispatch<F, E>(&mut self, cx: &mut Context, emit: F) -> Result<(), E>
    where
        F: FnOnce(&mut Context, (IpRepr, IcmpRepr)) -> Result<(), E>,
    {
        let hop_limit = self.hop_limit.unwrap_or(64);
        let res = self.tx_buffer.dequeue_with(|remote_endpoint, packet_buf| {
            /*
            net_trace!(
                "icmp:{}: sending {} octets",
                remote_endpoint,
                packet_buf.len()
            );
            */
            match *remote_endpoint {
                #[cfg(feature = "proto-ipv4")]
                IpAddress::Ipv4(dst_addr) => {
                    let src_addr = match cx.get_source_address_ipv4(&dst_addr) {
                        Some(addr) => addr,
                        None => {
                            net_trace!(
                                "icmp:{}: not find suitable source address, dropping",
                                remote_endpoint
                            );
                            return Ok(());
                        }
                    };
                    let packet = Icmpv4Packet::new_unchecked(&*packet_buf);
                    let repr = match Icmpv4Repr::parse(&packet, &ChecksumCapabilities::ignored()) {
                        Ok(x) => x,
                        Err(_) => {
                            net_trace!(
                                "icmp:{}: malformed packet in queue, dropping",
                                remote_endpoint
                            );
                            return Ok(());
                        }
                    };
                    let ip_repr = IpRepr::Ipv4(Ipv4Repr {
                        src_addr,
                        dst_addr,
                        next_header: IpProtocol::Icmp,
                        payload_len: repr.buffer_len(),
                        hop_limit,
                    });
                    emit(cx, (ip_repr, IcmpRepr::Ipv4(repr)))
                }
            }
        });
        match res {
            Err(Empty) => Ok(()),
            Ok(Err(e)) => Err(e),
            Ok(Ok(())) => {
                #[cfg(feature = "async")]
                self.tx_waker.wake();
                Ok(())
            }
        }
    }

    // 647
    pub(crate) fn poll_at(&self, _cx: &mut Context) -> PollAt {
        if self.tx_buffer.is_empty() {
            PollAt::Ingress
        } else {
            PollAt::Now
        }
    }
}
