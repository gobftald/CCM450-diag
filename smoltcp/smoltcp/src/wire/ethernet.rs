/// A six-octet Ethernet II address.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
// 28
pub struct Address(pub [u8; 6]);

// 82
#[cfg(feature = "defmt")]
impl defmt::Format for Address {
    fn format(&self, fmt: defmt::Formatter) {
        let bytes = self.0;
        defmt::write!(
            fmt,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5]
        )
    }
}
