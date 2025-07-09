#[cfg(feature = "proto-dhcpv4")]
// 82
pub(crate) mod dhcpv4;

#[cfg(feature = "medium-ethernet")]
// 86
mod ethernet;

// 98
pub(crate) mod ip;

#[cfg(feature = "proto-ipv4")]
// 100
pub(crate) mod ipv4;

// 140
use crate::phy::Medium;

#[cfg(feature = "medium-ethernet")]
// 144
pub use self::ethernet::Address as EthernetAddress;

// 181
pub use self::ip::{Address as IpAddress, Cidr as IpCidr};

// 187
#[cfg(feature = "proto-ipv4")]
pub use self::ipv4::{Address as Ipv4Address, Cidr as Ipv4Cidr};

#[cfg(feature = "proto-ipv4")]
// 196
pub(crate) use self::ipv4::AddressExt as Ipv4AddressExt;

#[cfg(feature = "proto-dhcpv4")]
// 281
pub use self::dhcpv4::{
    DhcpOption, Packet as DhcpPacket, CLIENT_PORT as DHCP_CLIENT_PORT,
    MAX_DNS_SERVER_COUNT as DHCP_MAX_DNS_SERVER_COUNT, SERVER_PORT as DHCP_SERVER_PORT,
};

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
