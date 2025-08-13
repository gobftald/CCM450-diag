// 1
use crate::phy::DeviceCapabilities;
use crate::wire::*;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "medium-ethernet")]
// 8
pub(crate) enum EthernetPacket<'a> {
    #[cfg(feature = "proto-ipv4")]
    Arp(ArpRepr),
    Ip(Packet<'a>),
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 16
pub(crate) enum Packet<'p> {
    #[cfg(feature = "proto-ipv4")]
    Ipv4(PacketV4<'p>),
    #[cfg(feature = "proto-ipv6")]
    Ipv6(PacketV6<'p>),
}

// 23
impl<'p> Packet<'p> {
    // 24
    pub(crate) fn new(ip_repr: IpRepr, payload: IpPayload<'p>) -> Self {
        match ip_repr {
            #[cfg(feature = "proto-ipv4")]
            IpRepr::Ipv4(header) => Self::new_ipv4(header, payload),
            #[cfg(feature = "proto-ipv6")]
            IpRepr::Ipv6(header) => Self::new_ipv6(header, payload),
        }
    }

    #[cfg(feature = "proto-ipv4")]
    // 34
    pub(crate) fn new_ipv4(ip_repr: Ipv4Repr, payload: IpPayload<'p>) -> Self {
        Self::Ipv4(PacketV4 {
            header: ip_repr,
            payload,
        })
    }

    // 55
    pub(crate) fn ip_repr(&self) -> IpRepr {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Packet::Ipv4(p) => IpRepr::Ipv4(p.header),
            #[cfg(feature = "proto-ipv6")]
            Packet::Ipv6(p) => IpRepr::Ipv6(p.header),
        }
    }

    // 64
    pub(crate) fn payload(&self) -> &IpPayload<'p> {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Packet::Ipv4(p) => &p.payload,
            #[cfg(feature = "proto-ipv6")]
            Packet::Ipv6(p) => &p.payload,
        }
    }

    // 73
    pub(crate) fn emit_payload(
        &self,
        _ip_repr: &IpRepr,
        payload: &mut [u8],
        caps: &DeviceCapabilities,
    ) {
        match self.payload() {
            #[cfg(feature = "proto-ipv4")]
            IpPayload::Icmpv4(icmpv4_repr) => {
                icmpv4_repr.emit(&mut Icmpv4Packet::new_unchecked(payload), &caps.checksum)
            }
            //#[cfg(any(feature = "socket-udp", feature = "socket-dns"))]
            #[cfg(feature = "socket-udp")]
            IpPayload::Udp(udp_repr, inner_payload) => udp_repr.emit(
                &mut UdpPacket::new_unchecked(payload),
                &_ip_repr.src_addr(),
                &_ip_repr.dst_addr(),
                inner_payload.len(),
                |buf| buf.copy_from_slice(inner_payload),
                &caps.checksum,
            ),
            #[cfg(feature = "socket-dhcpv4")]
            IpPayload::Dhcpv4(udp_repr, dhcp_repr) => udp_repr.emit(
                &mut UdpPacket::new_unchecked(payload),
                &_ip_repr.src_addr(),
                &_ip_repr.dst_addr(),
                dhcp_repr.buffer_len(),
                |buf| unwrap!(dhcp_repr.emit(&mut DhcpPacket::new_unchecked(buf))),
                &caps.checksum,
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "proto-ipv4")]
// 186
pub(crate) struct PacketV4<'p> {
    header: Ipv4Repr,
    payload: IpPayload<'p>,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 207
pub(crate) enum IpPayload<'p> {
    #[cfg(feature = "proto-ipv4")]
    Icmpv4(Icmpv4Repr<'p>),
    //#[cfg(all(feature = "proto-ipv4", feature = "multicast"))]
    //Igmp(IgmpRepr),
    #[cfg(feature = "proto-ipv6")]
    Icmpv6(Icmpv6Repr<'p>),
    #[cfg(feature = "proto-ipv6")]
    HopByHopIcmpv6(Ipv6HopByHopRepr<'p>, Icmpv6Repr<'p>),
    //#[cfg(feature = "socket-raw")]
    //Raw(&'p [u8]),
    //#[cfg(any(feature = "socket-udp", feature = "socket-dns"))]
    #[cfg(feature = "socket-udp")]
    Udp(UdpRepr, &'p [u8]),
    //#[cfg(feature = "socket-tcp")]
    //Tcp(TcpRepr<'p>),
    #[cfg(feature = "socket-dhcpv4")]
    Dhcpv4(UdpRepr, DhcpRepr<'p>),
}

#[cfg(any(feature = "proto-ipv4", feature = "proto-ipv6"))]
// 251
pub(crate) fn icmp_reply_payload_len(len: usize, mtu: usize, header_len: usize) -> usize {
    // Send back as much of the original payload as will fit within
    // the minimum MTU required by IPv4. See RFC 1812 § 4.3.2.3 for
    // more details.
    //
    // Since the entire network layer packet must fit within the minimum
    // MTU supported, the payload must not exceed the following:
    //
    // <min mtu> - IP Header Size * 2 - ICMPv4 DstUnreachable hdr size
    len.min(mtu - header_len * 2 - 8)
}
