#![no_std]

// 38
use allocator_api2::vec::Vec;

// 47
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4::RetryConfig;

#[cfg(feature = "proto-ipv4")]
// 59
pub use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

/// Static IP address configuration.
#[cfg(feature = "proto-ipv4")]
#[derive(Debug, Clone, PartialEq, Eq)]
// 109
pub struct StaticConfigV4 {
    /// IP address and subnet mask.
    pub address: Ipv4Cidr,
    /// Default gateway.
    pub gateway: Option<Ipv4Address>,
    /// DNS servers.
    //pub dns_servers: Vec<Ipv4Address, 3>,
    pub dns_servers: Vec<Ipv4Address>,
}

/// DHCP configuration.
#[cfg(feature = "dhcpv4")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DhcpConfig {
    /// Maximum lease duration.
    ///
    /// If not set, the lease duration specified by the server will be used.
    /// If set, the lease duration will be capped at this value.
    pub max_lease_duration: Option<embassy_time::Duration>,
    /// Retry configuration.
    pub retry_config: RetryConfig,
    /// Ignore NAKs from DHCP servers.
    ///
    /// This is not compliant with the DHCP RFCs, since theoretically we must stop using the assigned IP when receiving a NAK. This can increase reliability on broken networks with buggy routers or rogue DHCP servers, however.
    pub ignore_naks: bool,
    /// Server port. This is almost always 67. Do not change unless you know what you're doing.
    pub server_port: u16,
    /// Client port. This is almost always 68. Do not change unless you know what you're doing.
    pub client_port: u16,
    /// Our hostname. This will be sent to the DHCP server as Option 12.
    #[cfg(feature = "dhcpv4-hostname")]
    pub hostname: Option<heapless::String<MAX_HOSTNAME_LEN>>,
}

// 173
/// Network stack configuration.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Config {
    /// IPv4 configuration
    #[cfg(feature = "proto-ipv4")]
    pub ipv4: ConfigV4,
    /// IPv6 configuration
    #[cfg(feature = "proto-ipv6")]
    pub ipv6: ConfigV6,
}

#[cfg(feature = "dhcpv4")]
impl Default for DhcpConfig {
    fn default() -> Self {
        Self {
            max_lease_duration: Default::default(),
            retry_config: Default::default(),
            ignore_naks: Default::default(),
            server_port: smoltcp::wire::DHCP_SERVER_PORT,
            client_port: smoltcp::wire::DHCP_CLIENT_PORT,
            #[cfg(feature = "dhcpv4-hostname")]
            hostname: None,
        }
    }
}

// 182
impl Config {
    /// IPv4 configuration with static addressing.
    #[cfg(feature = "proto-ipv4")]
    // 185
    pub const fn ipv4_static(config: StaticConfigV4) -> Self {
        Self {
            ipv4: ConfigV4::Static(config),
            #[cfg(feature = "proto-ipv6")]
            ipv6: ConfigV6::None,
        }
    }

    /// IPv4 configuration with dynamic addressing.
    ///
    #[cfg(feature = "dhcpv4")]
    // 211
    pub const fn dhcpv4(config: DhcpConfig) -> Self {
        Self {
            ipv4: ConfigV4::Dhcp(config),
            #[cfg(feature = "proto-ipv6")]
            ipv6: ConfigV6::None,
        }
    }
}

/// Network stack IPv4 configuration.
#[cfg(feature = "proto-ipv4")]
#[derive(Debug, Clone, Default)]
// 223
pub enum ConfigV4 {
    /// Do not configure IPv4.
    #[default]
    None,
    /// Use a static IPv4 address configuration.
    Static(StaticConfigV4),
    /// Use DHCP to obtain an IP address configuration.
    #[cfg(feature = "dhcpv4")]
    Dhcp(DhcpConfig),
}
