#[cfg(feature = "proto-dhcpv4")]
// 82
pub(crate) mod dhcpv4;

#[cfg(feature = "proto-ipv4")]
// 100
pub(crate) mod ipv4;

// 187
#[cfg(feature = "proto-ipv4")]
pub use self::ipv4::{Address as Ipv4Address, Cidr as Ipv4Cidr};

#[cfg(feature = "proto-dhcpv4")]
// 281
pub use self::dhcpv4::{CLIENT_PORT as DHCP_CLIENT_PORT, SERVER_PORT as DHCP_SERVER_PORT};
