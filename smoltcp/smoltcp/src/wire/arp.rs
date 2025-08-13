// 1
use byteorder::{ByteOrder, NetworkEndian};

// 4
use super::{Error, Result};
use super::{EthernetAddress, Ipv4Address, Ipv4AddressExt};

// 7
pub use super::EthernetProtocol as Protocol;

// 9
enum_with_unknown! {
    /// ARP hardware type.
    pub enum Hardware(u16) {
        Ethernet = 1
    }
}

// 17
enum_with_unknown! {
    /// ARP operation type.
    pub enum Operation(u16) {
        Request = 1,
        Reply = 2
    }
}

/// A read/write wrapper around an Address Resolution Protocol packet buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 27
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

// 31
mod field {
    #![allow(non_snake_case)]

    use crate::wire::field::*;

    pub const HTYPE: Field = 0..2;
    pub const PTYPE: Field = 2..4;
    pub const HLEN: usize = 4;
    pub const PLEN: usize = 5;
    pub const OPER: Field = 6..8;

    #[inline]
    // 43
    pub const fn SHA(hardware_len: u8, _protocol_len: u8) -> Field {
        let start = OPER.end;
        start..(start + hardware_len as usize)
    }

    #[inline]
    // 49
    pub const fn SPA(hardware_len: u8, protocol_len: u8) -> Field {
        let start = SHA(hardware_len, protocol_len).end;
        start..(start + protocol_len as usize)
    }

    #[inline]
    // 55
    pub const fn THA(hardware_len: u8, protocol_len: u8) -> Field {
        let start = SPA(hardware_len, protocol_len).end;
        start..(start + hardware_len as usize)
    }

    #[inline]
    // 61
    pub const fn TPA(hardware_len: u8, protocol_len: u8) -> Field {
        let start = THA(hardware_len, protocol_len).end;
        start..(start + protocol_len as usize)
    }
}

// 67
impl<T: AsRef<[u8]>> Packet<T> {
    // 69
    pub const fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    /// Shorthand for a combination of [new_unchecked] and [check_len].
    ///
    /// [new_unchecked]: #method.new_unchecked
    /// [check_len]: #method.check_len
    // 77
    pub fn new_checked(buffer: T) -> Result<Packet<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    /// Ensure that no accessor method will panic if called.
    /// Returns `Err(Error)` if the buffer is too short.
    ///
    /// The result of this check is invalidated by calling [set_hardware_len] or
    /// [set_protocol_len].
    ///
    /// [set_hardware_len]: #method.set_hardware_len
    /// [set_protocol_len]: #method.set_protocol_len
    #[allow(clippy::if_same_then_else)]
    // 92
    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < field::OPER.end {
            Err(Error)
        } else if len < field::TPA(self.hardware_len(), self.protocol_len()).end {
            Err(Error)
        } else {
            Ok(())
        }
    }

    /// Return the hardware type field.
    #[inline]
    // 110
    pub fn hardware_type(&self) -> Hardware {
        let data = self.buffer.as_ref();
        let raw = NetworkEndian::read_u16(&data[field::HTYPE]);
        Hardware::from(raw)
    }

    /// Return the protocol type field.
    #[inline]
    // 118
    pub fn protocol_type(&self) -> Protocol {
        let data = self.buffer.as_ref();
        let raw = NetworkEndian::read_u16(&data[field::PTYPE]);
        Protocol::from(raw)
    }

    /// Return the hardware length field.
    #[inline]
    // 126
    pub fn hardware_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::HLEN]
    }

    /// Return the protocol length field.
    #[inline]
    // 133
    pub fn protocol_len(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::PLEN]
    }

    /// Return the operation field.
    #[inline]
    // 140
    pub fn operation(&self) -> Operation {
        let data = self.buffer.as_ref();
        let raw = NetworkEndian::read_u16(&data[field::OPER]);
        Operation::from(raw)
    }

    /// Return the source hardware address field.
    // 147
    pub fn source_hardware_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::SHA(self.hardware_len(), self.protocol_len())]
    }

    /// Return the source protocol address field.
    // 153
    pub fn source_protocol_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::SPA(self.hardware_len(), self.protocol_len())]
    }

    /// Return the target hardware address field.
    // 159
    pub fn target_hardware_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::THA(self.hardware_len(), self.protocol_len())]
    }

    /// Return the target protocol address field.
    // 165
    pub fn target_protocol_addr(&self) -> &[u8] {
        let data = self.buffer.as_ref();
        &data[field::TPA(self.hardware_len(), self.protocol_len())]
    }
}

// 171
impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    /// Set the hardware type field.
    #[inline]
    // 174
    pub fn set_hardware_type(&mut self, value: Hardware) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::HTYPE], value.into())
    }

    /// Set the protocol type field.
    #[inline]
    // 181
    pub fn set_protocol_type(&mut self, value: Protocol) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::PTYPE], value.into())
    }

    /// Set the hardware length field.
    #[inline]
    // 188
    pub fn set_hardware_len(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::HLEN] = value
    }

    /// Set the protocol length field.
    #[inline]
    // 195
    pub fn set_protocol_len(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::PLEN] = value
    }

    /// Set the operation field.
    #[inline]
    // 202
    pub fn set_operation(&mut self, value: Operation) {
        let data = self.buffer.as_mut();
        NetworkEndian::write_u16(&mut data[field::OPER], value.into())
    }

    /// Set the source hardware address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.hardware_len()` long.
    // 211
    pub fn set_source_hardware_addr(&mut self, value: &[u8]) {
        let (hardware_len, protocol_len) = (self.hardware_len(), self.protocol_len());
        let data = self.buffer.as_mut();
        data[field::SHA(hardware_len, protocol_len)].copy_from_slice(value)
    }

    /// Set the source protocol address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.protocol_len()` long.
    // 221
    pub fn set_source_protocol_addr(&mut self, value: &[u8]) {
        let (hardware_len, protocol_len) = (self.hardware_len(), self.protocol_len());
        let data = self.buffer.as_mut();
        data[field::SPA(hardware_len, protocol_len)].copy_from_slice(value)
    }

    /// Set the target hardware address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.hardware_len()` long.
    // 231
    pub fn set_target_hardware_addr(&mut self, value: &[u8]) {
        let (hardware_len, protocol_len) = (self.hardware_len(), self.protocol_len());
        let data = self.buffer.as_mut();
        data[field::THA(hardware_len, protocol_len)].copy_from_slice(value)
    }

    /// Set the target protocol address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.protocol_len()` long.
    // 241
    pub fn set_target_protocol_addr(&mut self, value: &[u8]) {
        let (hardware_len, protocol_len) = (self.hardware_len(), self.protocol_len());
        let data = self.buffer.as_mut();
        data[field::TPA(hardware_len, protocol_len)].copy_from_slice(value)
    }
}

/// A high-level representation of an Address Resolution Protocol packet.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 258
pub enum Repr {
    /// An Ethernet and IPv4 Address Resolution Protocol packet.
    EthernetIpv4 {
        operation: Operation,
        source_hardware_addr: EthernetAddress,
        source_protocol_addr: Ipv4Address,
        target_hardware_addr: EthernetAddress,
        target_protocol_addr: Ipv4Address,
    },
}

// 269
impl Repr {
    /// Parse an Address Resolution Protocol packet and return a high-level representation,
    /// or return `Err(Error)` if the packet is not recognized.
    // 272
    pub fn parse<T: AsRef<[u8]>>(packet: &Packet<T>) -> Result<Repr> {
        packet.check_len()?;

        match (
            packet.hardware_type(),
            packet.protocol_type(),
            packet.hardware_len(),
            packet.protocol_len(),
        ) {
            (Hardware::Ethernet, Protocol::Ipv4, 6, 4) => Ok(Repr::EthernetIpv4 {
                operation: packet.operation(),
                source_hardware_addr: EthernetAddress::from_bytes(packet.source_hardware_addr()),
                source_protocol_addr: Ipv4Address::from_bytes(packet.source_protocol_addr()),
                target_hardware_addr: EthernetAddress::from_bytes(packet.target_hardware_addr()),
                target_protocol_addr: Ipv4Address::from_bytes(packet.target_protocol_addr()),
            }),
            _ => Err(Error),
        }
    }

    /// Return the length of a packet that will be emitted from this high-level representation.
    // 293
    pub const fn buffer_len(&self) -> usize {
        match *self {
            Repr::EthernetIpv4 { .. } => field::TPA(6, 4).end,
        }
    }

    /// Emit a high-level representation into an Address Resolution Protocol packet.
    // 300
    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(&self, packet: &mut Packet<T>) {
        match *self {
            Repr::EthernetIpv4 {
                operation,
                source_hardware_addr,
                source_protocol_addr,
                target_hardware_addr,
                target_protocol_addr,
            } => {
                packet.set_hardware_type(Hardware::Ethernet);
                packet.set_protocol_type(Protocol::Ipv4);
                packet.set_hardware_len(6);
                packet.set_protocol_len(4);
                packet.set_operation(operation);
                packet.set_source_hardware_addr(source_hardware_addr.as_bytes());
                packet.set_source_protocol_addr(&source_protocol_addr.octets());
                packet.set_target_hardware_addr(target_hardware_addr.as_bytes());
                packet.set_target_protocol_addr(&target_protocol_addr.octets());
            }
        }
    }
}
