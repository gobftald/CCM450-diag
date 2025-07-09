#![no_std]

/// Representation of an hardware address, such as an Ethernet address or an IEEE802.15.4 address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum HardwareAddress {
    /// Ethernet medium, with a A six-octet Ethernet address.
    ///
    /// Devices of this type send and receive Ethernet frames,
    /// and interfaces using it must do neighbor discovery via ARP or NDISC.
    ///
    /// Examples of devices of this type are Ethernet, WiFi (802.11), Linux `tap`, and VPNs in tap (layer 2) mode.
    Ethernet([u8; 6]),
    /*
    /// 6LoWPAN over IEEE802.15.4, with an eight-octet address.
    Ieee802154([u8; 8]),
    /// Indicates that a Driver is IP-native, and has no hardware address.
    ///
    /// Devices of this type send and receive IP frames, without an
    /// Ethernet header. MAC addresses are not used, and no neighbor discovery (ARP, NDISC) is done.
    ///
    /// Examples of devices of this type are the Linux `tun`, PPP interfaces, VPNs in tun (layer 3) mode.
    Ip,
    */
}

/// Main `embassy-net` driver API.
///
/// This is essentially an interface for sending and receiving raw network frames.
///
/// The interface is based on _tokens_, which are types that allow to receive/transmit a
/// single packet. The `receive` and `transmit` functions only construct such tokens, the
/// real sending/receiving operation are performed when the tokens are consumed.
// 37
pub trait Driver {
    /// Get a description of device capabilities.
    // 75
    fn capabilities(&self) -> Capabilities;

    /// Get the device's hardware address.
    ///
    /// The returned hardware address also determines the "medium" of this driver. This indicates
    /// what kind of packet the sent/received bytes are, and determines some behaviors of
    /// the interface. For example, ARP/NDISC address resolution is only done for Ethernet mediums.
    // 82
    fn hardware_address(&self) -> HardwareAddress;
}

// 85
impl<T: ?Sized + Driver> Driver for &mut T {
    // 101
    fn capabilities(&self) -> Capabilities {
        T::capabilities(self)
    }

    // 107
    fn hardware_address(&self) -> HardwareAddress {
        T::hardware_address(self)
    }
}

/// A description of device capabilities.
///
/// Higher-level protocols may achieve higher throughput or lower latency if they consider
/// the bandwidth or packet size limitations.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 143
pub struct Capabilities {
    /// Maximum transmission unit.
    ///
    /// The network device is unable to send or receive frames larger than the value returned
    /// by this function.
    ///
    /// For Ethernet devices, this is the maximum Ethernet frame size, including the Ethernet header (14 octets), but
    /// *not* including the Ethernet FCS (4 octets). Therefore, Ethernet MTU = IP MTU + 14.
    ///
    /// Note that in Linux and other OSes, "MTU" is the IP MTU, not the Ethernet MTU, even for Ethernet
    /// devices. This is a common source of confusion.
    ///
    /// Most common IP MTU is 1500. Minimum is 576 (for IPv4) or 1280 (for IPv6). Maximum is 9216 octets.
    pub max_transmission_unit: usize,

    /// Maximum burst size, in terms of MTU.
    ///
    /// The network device is unable to send or receive bursts large than the value returned
    /// by this function.
    ///
    /// If `None`, there is no fixed limit on burst size, e.g. if network buffers are
    /// dynamically allocated.
    pub max_burst_size: Option<usize>,

    /// Checksum behavior.
    ///
    /// If the network device is capable of verifying or computing checksums for some protocols,
    /// it can request that the stack not do so in software to improve performance.
    pub checksum: ChecksumCapabilities,
}

/// A description of checksum behavior for every supported protocol.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 178
pub struct ChecksumCapabilities {
    /// Checksum behavior for IPv4.
    pub ipv4: Checksum,
    /// Checksum behavior for UDP.
    pub udp: Checksum,
    /// Checksum behavior for TCP.
    pub tcp: Checksum,
    /// Checksum behavior for ICMPv4.
    pub icmpv4: Checksum,
    /// Checksum behavior for ICMPv6.
    pub icmpv6: Checksum,
}

/// A description of checksum behavior for a particular protocol.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]

// 194
pub enum Checksum {
    /// Verify checksum when receiving and compute checksum when sending.
    #[default]
    Both,
    /// Verify checksum when receiving.
    Rx,
    /// Compute checksum before sending.
    Tx,
    /// Ignore checksum completely.
    None,
}
