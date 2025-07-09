/// A description of checksum behavior for a particular protocol.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 172
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

/// A description of checksum behavior for every supported protocol.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 206
pub struct ChecksumCapabilities {
    pub ipv4: Checksum,
    pub udp: Checksum,
    pub tcp: Checksum,
    #[cfg(feature = "proto-ipv4")]
    pub icmpv4: Checksum,
    #[cfg(feature = "proto-ipv6")]
    pub icmpv6: Checksum,
}

/// A description of device capabilities.
///
/// Higher-level protocols may achieve higher throughput or lower latency if they consider
/// the bandwidth or packet size limitations.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 239
pub struct DeviceCapabilities {
    /// Medium of the device.
    ///
    /// This indicates what kind of packet the sent/received bytes are, and determines
    /// some behaviors of Interface. For example, ARP/NDISC address resolution is only done
    /// for Ethernet mediums.
    pub medium: Medium,

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

/// Type of medium of a device.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 295
pub enum Medium {
    /// Ethernet medium. Devices of this type send and receive Ethernet frames,
    /// and interfaces using it must do neighbor discovery via ARP or NDISC.
    ///
    /// Examples of devices of this type are Ethernet, WiFi (802.11), Linux `tap`, and VPNs in tap (layer 2) mode.
    #[cfg(feature = "medium-ethernet")]
    Ethernet,
    /*
    /// IP medium. Devices of this type send and receive IP frames, without an
    /// Ethernet header. MAC addresses are not used, and no neighbor discovery (ARP, NDISC) is done.
    ///
    /// Examples of devices of this type are the Linux `tun`, PPP interfaces, VPNs in tun (layer 3) mode.
    #[cfg(feature = "medium-ip")]
    Ip,

    #[cfg(feature = "medium-ieee802154")]
    Ieee802154,
    */
}

// 314
impl Default for Medium {
    fn default() -> Medium {
        #[cfg(feature = "medium-ethernet")]
        return Medium::Ethernet;
        /*
        #[cfg(all(feature = "medium-ip", not(feature = "medium-ethernet")))]
        return Medium::Ip;
        #[cfg(all(
            feature = "medium-ieee802154",
            not(feature = "medium-ip"),
            not(feature = "medium-ethernet")
        ))]
        return Medium::Ieee802154;
        #[cfg(all(
            not(feature = "medium-ip"),
            not(feature = "medium-ethernet"),
            not(feature = "medium-ieee802154")
        ))]
        return panic!("No medium enabled");
        */
    }
}

/// An interface for sending and receiving raw network frames.
///
/// The interface is based on _tokens_, which are types that allow to receive/transmit a
/// single packet. The `receive` and `transmit` functions only construct such tokens, the
/// real sending/receiving operation are performed when the tokens are consumed.
// 340
pub trait Device {
    /// Get a description of device capabilities.
    // 366
    fn capabilities(&self) -> DeviceCapabilities;
}
