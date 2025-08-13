// 73
mod field {
    pub type Field = ::core::ops::Range<usize>;
    pub type Rest = ::core::ops::RangeFrom<usize>;
}

#[cfg(all(feature = "proto-ipv4", feature = "medium-ethernet"))]
// 81
mod arp;

#[cfg(feature = "proto-dhcpv4")]
// 83
pub(crate) mod dhcpv4;

#[cfg(feature = "medium-ethernet")]
// 87
mod ethernet;

#[cfg(feature = "proto-ipv4")]
// 91
mod icmpv4;

// 98
pub(crate) mod ip;

#[cfg(feature = "proto-ipv4")]
// 100
pub(crate) mod ipv4;

// 130
mod udp;

// 140
use crate::phy::Medium;

#[cfg(feature = "medium-ethernet")]
// 145
pub use self::ethernet::{
    Address as EthernetAddress, EtherType as EthernetProtocol, Frame as EthernetFrame,
};

#[cfg(all(feature = "proto-ipv4", feature = "medium-ethernet"))]
// 151
pub use self::arp::{Operation as ArpOperation, Packet as ArpPacket, Repr as ArpRepr};

// 181
pub use self::ip::{
    Address as IpAddress, Cidr as IpCidr, Endpoint as IpEndpoint,
    ListenEndpoint as IpListenEndpoint, Protocol as IpProtocol, Repr as IpRepr,
    Version as IpVersion,
};

#[cfg(feature = "proto-ipv4")]
// 188
pub use self::ipv4::{
    Address as Ipv4Address, Cidr as Ipv4Cidr, Packet as Ipv4Packet, Repr as Ipv4Repr,
    MIN_MTU as IPV4_MIN_MTU, MULTICAST_ALL_SYSTEMS as IPV4_MULTICAST_ALL_SYSTEMS,
};

#[cfg(feature = "proto-ipv4")]
// 196
pub(crate) use self::ipv4::AddressExt as Ipv4AddressExt;

#[cfg(feature = "proto-ipv4")]
// 231
pub use self::icmpv4::{
    DstUnreachable as Icmpv4DstUnreachable, Packet as Icmpv4Packet, Repr as Icmpv4Repr,
};

// 273
pub use self::udp::{Packet as UdpPacket, Repr as UdpRepr, HEADER_LEN as UDP_HEADER_LEN};

#[cfg(feature = "proto-dhcpv4")]
// 281
pub use self::dhcpv4::{
    DhcpOption, MessageType as DhcpMessageType, Packet as DhcpPacket, Repr as DhcpRepr,
    CLIENT_PORT as DHCP_CLIENT_PORT, MAX_DNS_SERVER_COUNT as DHCP_MAX_DNS_SERVER_COUNT,
    SERVER_PORT as DHCP_SERVER_PORT,
};

/// Parsing a packet failed.
///
/// Either it is malformed, or it is not supported by smoltcp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 305
pub struct Error;

// 316
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 326
pub enum HardwareAddress {
    //#[cfg(feature = "medium-ip")]
    //Ip,
    #[cfg(feature = "medium-ethernet")]
    Ethernet(EthernetAddress),
    //#[cfg(feature = "medium-ieee802154")]
    //Ieee802154(Ieee802154Address),
}

#[cfg(feature = "medium-ethernet")]
// 364
impl HardwareAddress {
    /// Query whether the address is an unicast address.
    // 377
    pub fn is_unicast(&self) -> bool {
        match self {
            //#[cfg(feature = "medium-ip")]
            //HardwareAddress::Ip => unreachable!(),
            #[cfg(feature = "medium-ethernet")]
            HardwareAddress::Ethernet(addr) => addr.is_unicast(),
            //#[cfg(feature = "medium-ieee802154")]
            //HardwareAddress::Ieee802154(addr) => addr.is_unicast(),
        }
    }

    #[cfg(feature = "medium-ethernet")]
    // 401
    pub(crate) fn ethernet_or_panic(&self) -> EthernetAddress {
        match self {
            HardwareAddress::Ethernet(addr) => *addr,
            #[allow(unreachable_patterns)]
            _ => panic!("HardwareAddress is not Ethernet."),
        }
    }

    #[inline]
    // 419
    pub(crate) fn medium(&self) -> Medium {
        match self {
            //#[cfg(feature = "medium-ip")]
            //HardwareAddress::Ip => Medium::Ip,
            #[cfg(feature = "medium-ethernet")]
            HardwareAddress::Ethernet(_) => Medium::Ethernet,
            //#[cfg(feature = "medium-ieee802154")]
            //HardwareAddress::Ieee802154(_) => Medium::Ieee802154,
        }
    }
}

#[cfg(feature = "medium-ethernet")]
// 436
impl core::fmt::Display for HardwareAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            #[cfg(feature = "medium-ethernet")]
            HardwareAddress::Ethernet(addr) => write!(f, "{addr}"),
        }
    }
}

#[cfg(feature = "medium-ethernet")]
// 450
impl From<EthernetAddress> for HardwareAddress {
    fn from(addr: EthernetAddress) -> Self {
        HardwareAddress::Ethernet(addr)
    }
}
