// 2
use core::fmt;

// 5
use crate::phy::ChecksumCapabilities;

#[cfg(feature = "proto-ipv4")]
// 7
use crate::wire::{Ipv4Address, Ipv4AddressExt, Ipv4Cidr, Ipv4Packet, Ipv4Repr};

/// Internet protocol version.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 14
pub enum Version {
    #[cfg(feature = "proto-ipv4")]
    Ipv4,
    #[cfg(feature = "proto-ipv6")]
    Ipv6,
}

// 48
enum_with_unknown! {
    /// IP datagram encapsulated protocol.
    pub enum Protocol(u8) {
        HopByHop  = 0x00,
        Icmp      = 0x01,
        Igmp      = 0x02,
        Tcp       = 0x06,
        Udp       = 0x11,
        Ipv6Route = 0x2b,
        Ipv6Frag  = 0x2c,
        IpSecEsp  = 0x32,
        IpSecAh   = 0x33,
        Icmpv6    = 0x3a,
        Ipv6NoNxt = 0x3b,
        Ipv6Opts  = 0x3c
    }
}

/// An internetworking address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// 88
pub enum Address {
    /// An IPv4 address.
    #[cfg(feature = "proto-ipv4")]
    Ipv4(Ipv4Address),
    /// An IPv6 address.
    #[cfg(feature = "proto-ipv6")]
    Ipv6(Ipv6Address),
}

// 97
impl Address {
    /// Query whether the address is a valid unicast address.
    // 131
    pub fn is_unicast(&self) -> bool {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Address::Ipv4(addr) => addr.x_is_unicast(),
            #[cfg(feature = "proto-ipv6")]
            Address::Ipv6(addr) => addr.x_is_unicast(),
        }
    }

    /// Query whether the address is a valid multicast address.
    // 141
    pub const fn is_multicast(&self) -> bool {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Address::Ipv4(addr) => addr.is_multicast(),
            #[cfg(feature = "proto-ipv6")]
            Address::Ipv6(addr) => addr.is_multicast(),
        }
    }

    /// Query whether the address is the broadcast address.
    // 151
    pub fn is_broadcast(&self) -> bool {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Address::Ipv4(addr) => addr.is_broadcast(),
            #[cfg(feature = "proto-ipv6")]
            Address::Ipv6(_) => false,
        }
    }

    /// Query whether the address falls into the "unspecified" range.
    // 161
    pub fn is_unspecified(&self) -> bool {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Address::Ipv4(addr) => addr.is_unspecified(),
            #[cfg(feature = "proto-ipv6")]
            Address::Ipv6(addr) => addr.is_unspecified(),
        }
    }

    /// If `self` is a CIDR-compatible subnet mask, return `Some(prefix_len)`,
    /// where `prefix_len` is the number of leading zeroes. Return `None` otherwise.
    // 172
    pub fn prefix_len(&self) -> Option<u8> {
        match self {
            #[cfg(feature = "proto-ipv4")]
            Address::Ipv4(addr) => addr.prefix_len(),
            #[cfg(feature = "proto-ipv6")]
            Address::Ipv6(addr) => addr.prefix_len(),
        }
    }
}

#[cfg(feature = "proto-ipv4")]
// 204
impl From<Ipv4Address> for Address {
    fn from(ipv4: Ipv4Address) -> Address {
        Address::Ipv4(ipv4)
    }
}

// 217
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Address::Ipv4(addr) => write!(f, "{addr}"),
            #[cfg(feature = "proto-ipv6")]
            Address::Ipv6(addr) => write!(f, "{addr}"),
        }
    }
}

#[cfg(feature = "defmt")]
// 229
impl defmt::Format for Address {
    fn format(&self, f: defmt::Formatter) {
        match self {
            #[cfg(feature = "proto-ipv4")]
            &Address::Ipv4(addr) => defmt::write!(f, "{:?}", addr),
            #[cfg(feature = "proto-ipv6")]
            &Address::Ipv6(addr) => defmt::write!(f, "{:?}", addr),
        }
    }
}

/// A specification of a CIDR block, containing an address and a variable-length
/// subnet masking prefix length.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// 243
pub enum Cidr {
    #[cfg(feature = "proto-ipv4")]
    Ipv4(Ipv4Cidr),
    #[cfg(feature = "proto-ipv6")]
    Ipv6(Ipv6Cidr),
}

// 250
impl Cidr {
    /// Return the IP address of this CIDR block.
    // 265
    pub const fn address(&self) -> Address {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Cidr::Ipv4(cidr) => Address::Ipv4(cidr.address()),
            #[cfg(feature = "proto-ipv6")]
            Cidr::Ipv6(cidr) => Address::Ipv6(cidr.address()),
        }
    }

    /// Return the prefix length of this CIDR block.
    // 275
    pub const fn prefix_len(&self) -> u8 {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Cidr::Ipv4(cidr) => cidr.prefix_len(),
            #[cfg(feature = "proto-ipv6")]
            Cidr::Ipv6(cidr) => cidr.prefix_len(),
        }
    }

    /// Query whether the subnetwork described by this CIDR block contains
    /// the given address.
    // 286
    pub fn contains_addr(&self, addr: &Address) -> bool {
        match (self, addr) {
            #[cfg(feature = "proto-ipv4")]
            (Cidr::Ipv4(cidr), Address::Ipv4(addr)) => cidr.contains_addr(addr),
            #[cfg(feature = "proto-ipv6")]
            (Cidr::Ipv6(cidr), Address::Ipv6(addr)) => cidr.contains_addr(addr),
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

#[cfg(feature = "defmt")]
// 337
impl defmt::Format for Cidr {
    fn format(&self, f: defmt::Formatter) {
        match self {
            #[cfg(feature = "proto-ipv4")]
            &Cidr::Ipv4(cidr) => defmt::write!(f, "{:?}", cidr),
            #[cfg(feature = "proto-ipv6")]
            &Cidr::Ipv6(cidr) => defmt::write!(f, "{:?}", cidr),
        }
    }
}

/// An internet endpoint address.
///
/// `Endpoint` always fully specifies both the address and the port.
///
/// See also ['ListenEndpoint'], which allows not specifying the address
/// in order to listen on a given port on any address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// 355
pub struct Endpoint {
    pub addr: Address,
    pub port: u16,
}

#[cfg(feature = "defmt")]
// 404
impl defmt::Format for Endpoint {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{:?}:{=u16}", self.addr, self.port);
    }
}

/// An internet endpoint address for listening.
///
/// In contrast with [`Endpoint`], `ListenEndpoint` allows not specifying the address,
/// in order to listen on a given port at all our addresses.
///
/// An endpoint can be constructed from a port, in which case the address is unspecified.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// 426
pub struct ListenEndpoint {
    pub addr: Option<Address>,
    pub port: u16,
}

#[cfg(feature = "defmt")]
// 479
impl defmt::Format for ListenEndpoint {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{:?}:{=u16}", self.addr, self.port);
    }
}

/// An IP packet representation.
///
/// This enum abstracts the various versions of IP packets. It either contains an IPv4
/// or IPv6 concrete high-level representation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 515
pub enum Repr {
    #[cfg(feature = "proto-ipv4")]
    Ipv4(Ipv4Repr),
    #[cfg(feature = "proto-ipv6")]
    Ipv6(Ipv6Repr),
}

// 536
impl Repr {
    /// Create a new IpRepr, choosing the right IP version for the src/dst addrs.
    ///
    /// # Panics
    ///
    /// Panics if `src_addr` and `dst_addr` are different IP version.
    // 542
    pub fn new(
        src_addr: Address,
        dst_addr: Address,
        next_header: Protocol,
        payload_len: usize,
        hop_limit: u8,
    ) -> Self {
        match (src_addr, dst_addr) {
            #[cfg(feature = "proto-ipv4")]
            (Address::Ipv4(src_addr), Address::Ipv4(dst_addr)) => Self::Ipv4(Ipv4Repr {
                src_addr,
                dst_addr,
                next_header,
                payload_len,
                hop_limit,
            }),
            #[allow(unreachable_patterns)]
            //_ => panic!("IP version mismatch: src={src_addr:?} dst={dst_addr:?}"),
            _ => panic!("IP version mismatch"),
        }
    }

    /// Return the protocol version.
    // 572
    pub const fn version(&self) -> Version {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Repr::Ipv4(_) => Version::Ipv4,
            #[cfg(feature = "proto-ipv6")]
            Repr::Ipv6(_) => Version::Ipv6,
        }
    }

    /// Return the source address.
    // 582
    pub const fn src_addr(&self) -> Address {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Repr::Ipv4(repr) => Address::Ipv4(repr.src_addr),
            #[cfg(feature = "proto-ipv6")]
            Repr::Ipv6(repr) => Address::Ipv6(repr.src_addr),
        }
    }

    /// Return the destination address.
    // 592
    pub const fn dst_addr(&self) -> Address {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Repr::Ipv4(repr) => Address::Ipv4(repr.dst_addr),
            #[cfg(feature = "proto-ipv6")]
            Repr::Ipv6(repr) => Address::Ipv6(repr.dst_addr),
        }
    }

    /// Return the payload length.
    // 612
    pub const fn payload_len(&self) -> usize {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Repr::Ipv4(repr) => repr.payload_len,
            #[cfg(feature = "proto-ipv6")]
            Repr::Ipv6(repr) => repr.payload_len,
        }
    }

    /// Return the length of a header that will be emitted from this high-level representation.
    // 642
    pub const fn header_len(&self) -> usize {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Repr::Ipv4(repr) => repr.buffer_len(),
            #[cfg(feature = "proto-ipv6")]
            Repr::Ipv6(repr) => repr.buffer_len(),
        }
    }

    /// Emit this high-level representation into a buffer.
    // 652
    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        buffer: T,
        _checksum_caps: &ChecksumCapabilities,
    ) {
        match *self {
            #[cfg(feature = "proto-ipv4")]
            Repr::Ipv4(repr) => repr.emit(&mut Ipv4Packet::new_unchecked(buffer), _checksum_caps),
            #[cfg(feature = "proto-ipv6")]
            Repr::Ipv6(repr) => repr.emit(&mut Ipv6Packet::new_unchecked(buffer)),
        }
    }

    /// Return the total length of a packet that will be emitted from this
    /// high-level representation.
    ///
    /// This is the same as `repr.buffer_len() + repr.payload_len()`.
    // 669
    pub const fn buffer_len(&self) -> usize {
        self.header_len() + self.payload_len()
    }
}

// 674
pub mod checksum {
    use byteorder::{ByteOrder, NetworkEndian};

    use super::*;

    // 679
    const fn propagate_carries(word: u32) -> u16 {
        let sum = (word >> 16) + (word & 0xffff);
        ((sum >> 16) as u16) + (sum as u16)
    }

    /// Compute an RFC 1071 compliant checksum (without the final complement).
    // 685
    pub fn data(mut data: &[u8]) -> u16 {
        let mut accum = 0;

        // For each 32-byte chunk...
        const CHUNK_SIZE: usize = 32;
        while data.len() >= CHUNK_SIZE {
            let mut d = &data[..CHUNK_SIZE];
            // ... take by 2 bytes and sum them.
            while d.len() >= 2 {
                accum += NetworkEndian::read_u16(d) as u32;
                d = &d[2..];
            }

            data = &data[CHUNK_SIZE..];
        }

        // Sum the rest that does not fit the last 32-byte chunk,
        // taking by 2 bytes.
        while data.len() >= 2 {
            accum += NetworkEndian::read_u16(data) as u32;
            data = &data[2..];
        }

        // Add the last remaining odd byte, if any.
        if let Some(&value) = data.first() {
            accum += (value as u32) << 8;
        }

        propagate_carries(accum)
    }

    /// Combine several RFC 1071 compliant checksums.
    // 717
    pub fn combine(checksums: &[u16]) -> u16 {
        let mut accum: u32 = 0;
        for &word in checksums {
            accum += word as u32;
        }
        propagate_carries(accum)
    }

    #[cfg(feature = "proto-ipv4")]
    // 726
    pub fn pseudo_header_v4(
        src_addr: &Ipv4Address,
        dst_addr: &Ipv4Address,
        next_header: Protocol,
        length: u32,
    ) -> u16 {
        let mut proto_len = [0u8; 4];
        proto_len[1] = next_header.into();
        NetworkEndian::write_u16(&mut proto_len[2..4], length as u16);

        combine(&[
            data(&src_addr.octets()),
            data(&dst_addr.octets()),
            data(&proto_len[..]),
        ])
    }

    // 761
    pub fn pseudo_header(
        src_addr: &Address,
        dst_addr: &Address,
        next_header: Protocol,
        length: u32,
    ) -> u16 {
        match (src_addr, dst_addr) {
            #[cfg(feature = "proto-ipv4")]
            (Address::Ipv4(src_addr), Address::Ipv4(dst_addr)) => {
                pseudo_header_v4(src_addr, dst_addr, next_header, length)
            }
            #[cfg(feature = "proto-ipv6")]
            (Address::Ipv6(src_addr), Address::Ipv6(dst_addr)) => {
                pseudo_header_v6(src_addr, dst_addr, next_header, length)
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}
