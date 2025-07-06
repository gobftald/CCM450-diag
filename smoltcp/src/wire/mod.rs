#[cfg(feature = "proto-ipv4")]
// 100
pub(crate) mod ipv4;

// 187
#[cfg(feature = "proto-ipv4")]
pub use self::ipv4::{Address as Ipv4Address, Cidr as Ipv4Cidr};
