// 44
pub use core::net::Ipv4Addr as Address;

// 46
pub(crate) trait AddressExt {
    /// Query whether the address is an unicast address.
    ///
    /// `x_` prefix is to avoid a collision with the still-unstable method in `core::ip`.
    fn x_is_unicast(&self) -> bool;
}

// 63
impl AddressExt for Address {
    /// Query whether the address is an unicast address.
    fn x_is_unicast(&self) -> bool {
        !(self.is_broadcast() || self.is_multicast() || self.is_unspecified())
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
        assert!(prefix_len <= 32);
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
}

#[cfg(feature = "defmt")]
// 196
impl defmt::Format for Cidr {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}/{=u8}", self.address, self.prefix_len);
    }
}
