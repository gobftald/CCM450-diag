// 1
use byteorder::{ByteOrder, NetworkEndian};

// 4
use super::{Error, Result};
use crate::phy::ChecksumCapabilities;
use crate::wire::ip::checksum;
use crate::wire::{IpAddress, IpProtocol};

/// A read/write wrapper around an User Datagram Protocol packet buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
// 11
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

// 15
mod field {
    #![allow(non_snake_case)]

    use crate::wire::field::*;

    pub const SRC_PORT: Field = 0..2;
    pub const DST_PORT: Field = 2..4;
    pub const LENGTH: Field = 4..6;
    pub const CHECKSUM: Field = 6..8;

    pub const fn PAYLOAD(length: u16) -> Field {
        CHECKSUM.end..(length as usize)
    }
}

// 30
pub const HEADER_LEN: usize = field::CHECKSUM.end;

// 33
impl<T: AsRef<[u8]>> Packet<T> {
    /// Imbue a raw octet buffer with UDP packet structure.
    // 35
    pub const fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    /// Shorthand for a combination of [new_unchecked] and [check_len].
    ///
    /// [new_unchecked]: #method.new_unchecked
    /// [check_len]: #method.check_len
    // 43
    pub fn new_checked(buffer: T) -> Result<Packet<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    /// Ensure that no accessor method will panic if called.
    /// Returns `Err(Error)` if the buffer is too short.
    /// Returns `Err(Error)` if the length field has a value smaller
    /// than the header length.
    ///
    /// The result of this check is invalidated by calling [set_len].
    ///
    // 57
    pub fn check_len(&self) -> Result<()> {
        let buffer_len = self.buffer.as_ref().len();
        if buffer_len < HEADER_LEN {
            Err(Error)
        } else {
            let field_len = self.len() as usize;
            if buffer_len < field_len || field_len < HEADER_LEN {
                Err(Error)
            } else {
                Ok(())
            }
        }
    }

    /// Return the source port field.
    #[inline]
    // 78
    pub fn src_port(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::SRC_PORT])
    }

    /// Return the destination port field.
    #[inline]
    // 85
    pub fn dst_port(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::DST_PORT])
    }

    /// Return the length field.
    #[inline]
    // 92
    pub fn len(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::LENGTH])
    }

    /// Return the checksum field.
    #[inline]
    // 99
    pub fn checksum(&self) -> u16 {
        let data = self.buffer.as_ref();
        NetworkEndian::read_u16(&data[field::CHECKSUM])
    }

    /// Validate the packet checksum.
    ///
    /// # Panics
    /// This function panics unless `src_addr` and `dst_addr` belong to the same family,
    /// and that family is IPv4 or IPv6.
    ///
    /// # Fuzzing
    /// This function always returns `true` when fuzzing.
    // 129
    pub fn verify_checksum(&self, src_addr: &IpAddress, dst_addr: &IpAddress) -> bool {
        /*
        if cfg!(fuzzing) {
            return true;
        }
        */

        // From the RFC:
        // > An all zero transmitted checksum value means that the transmitter
        // > generated no checksum (for debugging or for higher level protocols
        // > that don't care).
        if self.checksum() == 0 {
            return true;
        }

        let data = self.buffer.as_ref();
        checksum::combine(&[
            checksum::pseudo_header(src_addr, dst_addr, IpProtocol::Udp, self.len() as u32),
            checksum::data(&data[..self.len() as usize]),
        ]) == !0
    }
}

// 150
impl<'a, T: AsRef<[u8]> + ?Sized> Packet<&'a T> {
    /// Return a pointer to the payload.
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let length = self.len();
        let data = self.buffer.as_ref();
        &data[field::PAYLOAD(length)]
    }
}

// 160
impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    /// Set the source port field.
    #[inline]
    // 163
    pub fn set_src_port(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::SRC_PORT], value)
    }

    /// Set the destination port field.
    #[inline]
    // 170
    pub fn set_dst_port(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::DST_PORT], value)
    }

    /// Set the length field.
    #[inline]
    // 177
    pub fn set_len(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::LENGTH], value)
    }

    /// Set the checksum field.
    #[inline]
    // 184
    pub fn set_checksum(&mut self, value: u16) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::CHECKSUM], value)
    }

    /// Compute and fill in the header checksum.
    ///
    /// # Panics
    /// This function panics unless `src_addr` and `dst_addr` belong to the same family,
    /// and that family is IPv4 or IPv6.
    // 194
    pub fn fill_checksum(&mut self, src_addr: &IpAddress, dst_addr: &IpAddress) {
        self.set_checksum(0);
        let checksum = {
            let data = self.buffer.as_ref();
            !checksum::combine(&[
                checksum::pseudo_header(src_addr, dst_addr, IpProtocol::Udp, self.len() as u32),
                checksum::data(&data[..self.len() as usize]),
            ])
        };
        // UDP checksum value of 0 means no checksum; if the checksum really is zero,
        // use all-ones, which indicates that the remote end must verify the checksum.
        // Arithmetically, RFC 1071 checksums of all-zeroes and all-ones behave identically,
        // so no action is necessary on the remote end.
        self.set_checksum(if checksum == 0 { 0xffff } else { checksum })
    }

    /// Return a mutable pointer to the payload.
    #[inline]
    // 212
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let length = self.len();
        let data = self.buffer.as_mut();
        &mut data[field::PAYLOAD(length)]
    }
}

/// A high-level representation of an User Datagram Protocol packet.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
// 227
pub struct Repr {
    pub src_port: u16,
    pub dst_port: u16,
}

// 232
impl Repr {
    /// Parse an User Datagram Protocol packet and return a high-level representation.
    // 234
    pub fn parse<T>(
        packet: &Packet<&T>,
        src_addr: &IpAddress,
        dst_addr: &IpAddress,
        checksum_caps: &ChecksumCapabilities,
    ) -> Result<Repr>
    where
        T: AsRef<[u8]> + ?Sized,
    {
        packet.check_len()?;

        // Destination port cannot be omitted (but source port can be).
        if packet.dst_port() == 0 {
            return Err(Error);
        }
        // Valid checksum is expected...
        if checksum_caps.udp.rx() && !packet.verify_checksum(src_addr, dst_addr) {
            match (src_addr, dst_addr) {
                // ... except on UDP-over-IPv4, where it can be omitted.
                #[cfg(feature = "proto-ipv4")]
                (&IpAddress::Ipv4(_), &IpAddress::Ipv4(_)) if packet.checksum() == 0 => (),
                _ => return Err(Error),
            }
        }

        Ok(Repr {
            src_port: packet.src_port(),
            dst_port: packet.dst_port(),
        })
    }

    /// Return the length of the packet header that will be emitted from this high-level representation.
    // 266
    pub const fn header_len(&self) -> usize {
        HEADER_LEN
    }

    /// Emit a high-level representation into an User Datagram Protocol packet.
    // 286
    pub fn emit<T>(
        &self,
        packet: &mut Packet<&mut T>,
        src_addr: &IpAddress,
        dst_addr: &IpAddress,
        payload_len: usize,
        emit_payload: impl FnOnce(&mut [u8]),
        checksum_caps: &ChecksumCapabilities,
    ) where
        T: AsRef<[u8]> + AsMut<[u8]> + ?Sized,
    {
        packet.set_src_port(self.src_port);
        packet.set_dst_port(self.dst_port);
        packet.set_len((HEADER_LEN + payload_len) as u16);
        emit_payload(packet.payload_mut());

        if checksum_caps.udp.tx() {
            packet.fill_checksum(src_addr, dst_addr)
        } else {
            // make sure we get a consistently zeroed checksum,
            // since implementations might rely on it
            packet.set_checksum(0);
        }
    }
}

#[cfg(feature = "defmt")]
// 326
impl<'a, T: AsRef<[u8]> + ?Sized> defmt::Format for Packet<&'a T> {
    fn format(&self, fmt: defmt::Formatter) {
        // Cannot use Repr::parse because we don't have the IP addresses.
        defmt::write!(
            fmt,
            "UDP src={} dst={} len={}",
            self.src_port(),
            self.dst_port(),
            self.payload().len()
        );
    }
}

#[cfg(feature = "defmt")]
// 346
impl defmt::Format for Repr {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "UDP src={} dst={}", self.src_port, self.dst_port);
    }
}
