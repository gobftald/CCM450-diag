//12
pub const SERVER_PORT: u16 = 67;
pub const CLIENT_PORT: u16 = 68;
pub const MAX_DNS_SERVER_COUNT: usize = 3;

/// A representation of a single DHCP option.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DhcpOption<'a> {
    pub kind: u8,
    pub data: &'a [u8],
}

/// A read/write wrapper around a Dynamic Host Configuration Protocol packet buffer.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 116
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}
