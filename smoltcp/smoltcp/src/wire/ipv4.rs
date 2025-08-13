// 1
use byteorder::{ByteOrder, NetworkEndian};

// 4
use super::{Error, Result};
use crate::phy::ChecksumCapabilities;
use crate::wire::ip::checksum;

// 8
pub use super::IpProtocol as Protocol;

/// Minimum MTU required of all links supporting IPv4. See [RFC 791 § 3.1].
///
// As a result, we can assume that every host we send packets to can
// accept a packet of the following size.
// 22
pub const MIN_MTU: usize = 576;

/// Size of IPv4 adderess in octets.
///
/// [RFC 8200 § 2]: https://www.rfc-editor.org/rfc/rfc791#section-3.2
// 27
pub const ADDR_SIZE: usize = 4;

/// All multicast-capable nodes
// 30
pub const MULTICAST_ALL_SYSTEMS: Address = Address::new(224, 0, 0, 1);

// 44
pub use core::net::Ipv4Addr as Address;

// 46
pub(crate) trait AddressExt {
    /// Construct an IPv4 address from a sequence of octets, in big-endian.
    ///
    /// # Panics
    /// The function panics if `data` is not four octets long.
    fn from_bytes(data: &[u8]) -> Self;

    /// Query whether the address is an unicast address.
    ///
    /// `x_` prefix is to avoid a collision with the still-unstable method in `core::ip`.
    fn x_is_unicast(&self) -> bool;

    /// If `self` is a CIDR-compatible subnet mask, return `Some(prefix_len)`,
    /// where `prefix_len` is the number of leading zeroes. Return `None` otherwise.
    fn prefix_len(&self) -> Option<u8>;
}

// 63
impl AddressExt for Address {
    // 64
    fn from_bytes(data: &[u8]) -> Address {
        let mut bytes = [0; ADDR_SIZE];
        bytes.copy_from_slice(data);
        Address::from_bits(u32::from_be_bytes(bytes))
    }

    /// Query whether the address is an unicast address.
    // 71
    fn x_is_unicast(&self) -> bool {
        !(self.is_broadcast() || self.is_multicast() || self.is_unspecified())
    }

    // 75
    fn prefix_len(&self) -> Option<u8> {
        let mut ones = true;
        let mut prefix_len = 0;
        for byte in self.octets() {
            let mut mask = 0x80;
            for _ in 0..8 {
                let one = byte & mask != 0;
                if ones {
                    // Expect 1s until first 0
                    if one {
                        prefix_len += 1;
                    } else {
                        ones = false;
                    }
                } else if one {
                    // 1 where 0 was expected
                    return None;
                }
                mask >>= 1;
            }
        }
        Some(prefix_len)
    }
}

/// A specification of an IPv4 CIDR block, containing an address and a variable-length
/// subnet masking prefix length.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// 103
pub struct Cidr {
    address: Address,
    prefix_len: u8,
}

// 108
impl Cidr {
    /// Create an IPv4 CIDR block from the given address and prefix length.
    ///
    /// # Panics
    /// This function panics if the prefix length is larger than 32.
    // 113
    pub const fn new(address: Address, prefix_len: u8) -> Cidr {
        core::assert!(prefix_len <= 32);
        Cidr {
            address,
            prefix_len,
        }
    }

    /// Return the address of this IPv4 CIDR block.
    // 134
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Return the prefix length of this IPv4 CIDR block.
    // 140
    pub const fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    /// Return the network mask of this IPv4 CIDR.
    // 145
    pub const fn netmask(&self) -> Address {
        if self.prefix_len == 0 {
            return Address::new(0, 0, 0, 0);
        }

        let number = 0xffffffffu32 << (32 - self.prefix_len);
        Address::from_bits(number)
    }

    /// Return the broadcast address of this IPv4 CIDR.
    // 155
    pub fn broadcast(&self) -> Option<Address> {
        let network = self.network();

        if network.prefix_len == 31 || network.prefix_len == 32 {
            return None;
        }

        let network_number = network.address.to_bits();
        let number = network_number | 0xffffffffu32 >> network.prefix_len;
        Some(Address::from_bits(number))
    }

    /// Return the network block of this IPv4 CIDR.
    // 168
    pub const fn network(&self) -> Cidr {
        Cidr {
            address: Address::from_bits(self.address.to_bits() & self.netmask().to_bits()),
            prefix_len: self.prefix_len,
        }
    }

    /// Query whether the subnetwork described by this IPv4 CIDR block contains
    /// the given address.
    // 177
    pub fn contains_addr(&self, addr: &Address) -> bool {
        self.address.to_bits() & self.netmask().to_bits()
            == addr.to_bits() & self.netmask().to_bits()
    }
}

#[cfg(feature = "defmt")]
// 196
impl defmt::Format for Cidr {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}/{=u8}", self.address, self.prefix_len);
    }
}

/// A read/write wrapper around an Internet Protocol version 4 packet buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 205
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

// 209
mod field {
    use crate::wire::field::*;

    pub const VER_IHL: usize = 0;
    pub const DSCP_ECN: usize = 1;
    pub const LENGTH: Field = 2..4;
    pub const IDENT: Field = 4..6;
    pub const FLG_OFF: Field = 6..8;
    pub const TTL: usize = 8;
    pub const PROTOCOL: usize = 9;
    pub const CHECKSUM: Field = 10..12;
    pub const SRC_ADDR: Field = 12..16;
    pub const DST_ADDR: Field = 16..20;
}

// 226
impl<T: AsRef<[u8]>> Packet<T> {
    /// Imbue a raw octet buffer with IPv4 packet structure.
    // 228
    pub const fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    /// Shorthand for a combination of [new_unchecked] and [check_len].
    ///
    // 236
    pub fn new_checked(buffer: T) -> Result<Packet<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    // 253
    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < field::DST_ADDR.end {
            Err(Error)
        } else if len < self.header_len() as usize {
            Err(Error)
        } else if self.header_len() as u16 > self.total_len() {
            Err(Error)
        } else if len < self.total_len() as usize {
            Err(Error)
        } else {
            Ok(())
        }
    }

    /// Consume the packet, returning the underlying buffer.
    // 269
    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Return the version field.
    #[inline]
    // 275
    pub fn version(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::VER_IHL] >> 4
    }

    /// Return the header length, in octets.
    #[inline]
    // 282
    pub fn header_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        (data[field::VER_IHL] & 0x0f) * 4
    }

    /// Return the total length field.
    #[inline]
    // 301
    pub fn total_len(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::LENGTH])
    }

    /// Return the time to live field.
    #[inline]
    // 336
    pub fn hop_limit(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::TTL]
    }

    /// Return the next_header (protocol) field.
    #[inline]
    // 343
    pub fn next_header(&self) -> Protocol {
        let data = self.buffer.as_ref();
        Protocol::from(data[field::PROTOCOL])
    }

    /// Return the source address field.
    #[inline]
    // 357
    pub fn src_addr(&self) -> Address {
        let data = self.buffer.as_ref();
        Address::from_bytes(&data[field::SRC_ADDR])
    }
    /// Return the destination address field.
    #[inline]
    // 364
    pub fn dst_addr(&self) -> Address {
        let data = self.buffer.as_ref();
        Address::from_bytes(&data[field::DST_ADDR])
    }

    /// Validate the header checksum.
    ///
    /// # Fuzzing
    /// This function always returns `true` when fuzzing.
    // 373
    pub fn verify_checksum(&self) -> bool {
        /*
        if cfg!(fuzzing) {
            return true;
        }
        */

        let data = self.buffer.as_ref();
        checksum::data(&data[..self.header_len() as usize]) == !0
    }
}

// 393
impl<'a, T: AsRef<[u8]> + ?Sized> Packet<&'a T> {
    /// Return a pointer to the payload.
    #[inline]
    // 396
    pub fn payload(&self) -> &'a [u8] {
        let range = self.header_len() as usize..self.total_len() as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }
}

// 403
impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    /// Set the version field.
    #[inline]
    // 406
    pub fn set_version(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::VER_IHL] = (data[field::VER_IHL] & !0xf0) | (value << 4);
    }

    /// Set the header length, in octets.
    #[inline]
    // 413
    pub fn set_header_len(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::VER_IHL] = (data[field::VER_IHL] & !0x0f) | ((value / 4) & 0x0f);
    }

    /// Set the Differential Services Code Point field.
    // 419
    pub fn set_dscp(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::DSCP_ECN] = (data[field::DSCP_ECN] & !0xfc) | (value << 2)
    }

    /// Set the Explicit Congestion Notification field.
    // 425
    pub fn set_ecn(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::DSCP_ECN] = (data[field::DSCP_ECN] & !0x03) | (value & 0x03)
    }

    /// Set the total length field.
    #[inline]
    // 432
    pub fn set_total_len(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::LENGTH], value)
    }

    /// Set the fragment identification field.
    #[inline]
    // 439
    pub fn set_ident(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::IDENT], value)
    }

    /// Clear the entire flags field.
    #[inline]
    // 446
    pub fn clear_flags(&mut self) {
        let data = self.buffer.as_mut();
        let raw = NetworkEndian::read_u16(&data[field::FLG_OFF]);
        let raw = raw & !0xe000;
        NetworkEndian::write_u16(&mut data[field::FLG_OFF], raw);
    }

    /// Set the "don't fragment" flag.
    #[inline]
    // 455
    pub fn set_dont_frag(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        let raw = NetworkEndian::read_u16(&data[field::FLG_OFF]);
        let raw = if value { raw | 0x4000 } else { raw & !0x4000 };
        NetworkEndian::write_u16(&mut data[field::FLG_OFF], raw);
    }

    /// Set the "more fragments" flag.
    #[inline]
    // 464
    pub fn set_more_frags(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        let raw = NetworkEndian::read_u16(&data[field::FLG_OFF]);
        let raw = if value { raw | 0x2000 } else { raw & !0x2000 };
        NetworkEndian::write_u16(&mut data[field::FLG_OFF], raw);
    }

    /// Set the fragment offset, in octets.
    #[inline]
    // 473
    pub fn set_frag_offset(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        let raw = NetworkEndian::read_u16(&data[field::FLG_OFF]);
        let raw = (raw & 0xe000) | (value >> 3);
        NetworkEndian::write_u16(&mut data[field::FLG_OFF], raw);
    }

    /// Set the time to live field.
    #[inline]
    // 482
    pub fn set_hop_limit(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::TTL] = value
    }

    /// Set the next header (protocol) field.
    #[inline]
    // 489
    pub fn set_next_header(&mut self, value: Protocol) {
        let data = self.buffer.as_mut();
        data[field::PROTOCOL] = value.into()
    }

    /// Set the header checksum field.
    #[inline]
    // 496
    pub fn set_checksum(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::CHECKSUM], value)
    }

    /// Set the source address field.
    #[inline]
    // 503
    pub fn set_src_addr(&mut self, value: Address) {
        let data = self.buffer.as_mut();
        data[field::SRC_ADDR].copy_from_slice(&value.octets())
    }

    /// Set the destination address field.
    #[inline]
    // 510
    pub fn set_dst_addr(&mut self, value: Address) {
        let data = self.buffer.as_mut();
        data[field::DST_ADDR].copy_from_slice(&value.octets())
    }

    /// Compute and fill in the header checksum.
    // 516
    pub fn fill_checksum(&mut self) {
        self.set_checksum(0);
        let checksum = {
            let data = self.buffer.as_ref();
            !checksum::data(&data[..self.header_len() as usize])
        };
        self.set_checksum(checksum)
    }
}

/// A high-level representation of an Internet Protocol version 4 packet header.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 543
pub struct Repr {
    pub src_addr: Address,
    pub dst_addr: Address,
    pub next_header: Protocol,
    pub payload_len: usize,
    pub hop_limit: u8,
}

// 551
impl Repr {
    /// Parse an Internet Protocol version 4 packet and return a high-level representation.
    // 553
    pub fn parse<T: AsRef<[u8]> + ?Sized>(
        packet: &Packet<&T>,
        checksum_caps: &ChecksumCapabilities,
    ) -> Result<Repr> {
        packet.check_len()?;
        // Version 4 is expected.
        if packet.version() != 4 {
            return Err(Error);
        }
        // Valid checksum is expected.
        if checksum_caps.ipv4.rx() && !packet.verify_checksum() {
            return Err(Error);
        }

        let payload_len = packet.total_len() as usize - packet.header_len() as usize;

        // All DSCP ( Differentiated Services Code Point for managing network traffic for Quality of Service)
        // values are acceptable, since they are of no concern to receiving endpoint.
        // All ECN (Explicit Congestion Notification) values are acceptable, since ECN requires opt-in
        // from both endpoints.
        // All TTL (Time To Live) values are acceptable, since we do not perform routing.
        Ok(Repr {
            src_addr: packet.src_addr(),
            dst_addr: packet.dst_addr(),
            next_header: packet.next_header(),
            payload_len,
            hop_limit: packet.hop_limit(),
        })
    }

    /// Return the length of a header that will be emitted from this high-level representation.
    // 588
    pub const fn buffer_len(&self) -> usize {
        // We never emit any options.
        field::DST_ADDR.end
    }

    /// Emit a high-level representation into an Internet Protocol version 4 packet.
    // 594
    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        packet: &mut Packet<T>,
        checksum_caps: &ChecksumCapabilities,
    ) {
        packet.set_version(4);
        packet.set_header_len(field::DST_ADDR.end as u8);
        packet.set_dscp(0);
        packet.set_ecn(0);
        let total_len = packet.header_len() as u16 + self.payload_len as u16;
        packet.set_total_len(total_len);
        packet.set_ident(0);
        packet.clear_flags();
        packet.set_more_frags(false);
        packet.set_dont_frag(true);
        packet.set_frag_offset(0);
        packet.set_hop_limit(self.hop_limit);
        packet.set_next_header(self.next_header);
        packet.set_src_addr(self.src_addr);
        packet.set_dst_addr(self.dst_addr);

        if checksum_caps.ipv4.tx() {
            packet.fill_checksum();
        } else {
            // make sure we get a consistently zeroed checksum,
            // since implementations might rely on it
            packet.set_checksum(0);
        }
    }
}
