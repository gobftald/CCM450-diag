// 5
use crate::time::{Duration, Instant};

// 7
use crate::wire::{
    DhcpOption, DhcpPacket, Ipv4Address, Ipv4Cidr, DHCP_CLIENT_PORT, DHCP_MAX_DNS_SERVER_COUNT,
    DHCP_SERVER_PORT,
};

// 13
//use heapless::Vec;
use allocator_api2::vec::Vec;

#[cfg(feature = "async")]
// 16
use super::WakerRegistration;

/// IPv4 configuration data provided by the DHCP server.
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 31
pub struct Config<'a> {
    /// Information on how to reach the DHCP server that responded with DHCP
    /// configuration.
    pub server: ServerInfo,
    /// IP address
    pub address: Ipv4Cidr,
    /// Router address, also known as default gateway. Does not necessarily
    /// match the DHCP server's address.
    pub router: Option<Ipv4Address>,
    /// DNS servers
    // DHCP_MAX_DNS_SERVER_COUNT will be used when 'push'
    //pub dns_servers: Vec<Ipv4Address, DHCP_MAX_DNS_SERVER_COUNT>,
    pub dns_servers: Vec<Ipv4Address>,
    /// Received DHCP packet
    pub packet: Option<DhcpPacket<&'a [u8]>>,
}

/// Information on how to reach a DHCP server.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 49
pub struct ServerInfo {
    /// IP address to use as destination in outgoing packets
    pub address: Ipv4Address,
    /// Server identifier to use in outgoing packets. Usually equal to server_address,
    /// but may differ in some situations (eg DHCP relays)
    pub identifier: Ipv4Address,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 59
struct DiscoverState {
    /// When to send next request
    retry_at: Instant,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 66
struct RequestState {
    /// When to send next request
    retry_at: Instant,
    /// How many retries have been done
    retry: u16,
    /// Server we're trying to request from
    server: ServerInfo,
    /// IP address that we're trying to request.
    requested_ip: Ipv4Address,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 79
struct RenewState {
    /// Active network config
    config: Config<'static>,

    /// Renew timer. When reached, we will start attempting
    /// to renew this lease with the DHCP server.
    ///
    /// Must be less or equal than `rebind_at`.
    renew_at: Instant,

    /// Rebind timer. When reached, we will start broadcasting to renew
    /// this lease with any DHCP server.
    ///
    /// Must be greater than or equal to `renew_at`, and less than or
    /// equal to `expires_at`.
    rebind_at: Instant,

    /// Whether the T2 time has elapsed
    rebinding: bool,

    /// Expiration timer. When reached, this lease is no longer valid, so it must be
    /// thrown away and the ethernet interface deconfigured.
    expires_at: Instant,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 106
enum ClientState {
    /// Discovering the DHCP server
    Discovering(DiscoverState),
    /// Requesting an address
    Requesting(RequestState),
    /// Having an address, refresh it periodically.
    Renewing(RenewState),
}

/// Timeout and retry configuration.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 119
pub struct RetryConfig {
    pub discover_timeout: Duration,
    /// The REQUEST timeout doubles every 2 tries.
    pub initial_request_timeout: Duration,
    pub request_retries: u16,
    pub min_renew_timeout: Duration,
    /// An upper bound on how long to wait between retrying a renew or rebind.
    ///
    /// Set this to [`Duration::MAX`] if you don't want to impose an upper bound.
    pub max_renew_timeout: Duration,
}

// 131
impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            discover_timeout: Duration::from_secs(10),
            initial_request_timeout: Duration::from_secs(5),
            request_retries: 5,
            min_renew_timeout: Duration::from_secs(60),
            max_renew_timeout: Duration::MAX,
        }
    }
}

#[derive(Debug)]

// 154
pub struct Socket<'a> {
    /// State of the DHCP client.
    state: ClientState,
    /// Set to true on config/state change, cleared back to false by the `config` function.
    config_changed: bool,
    /// xid of the last sent message.
    transaction_id: u32,

    /// Max lease duration. If set, it sets a maximum cap to the server-provided lease duration.
    /// Useful to react faster to IP configuration changes and to test whether renews work correctly.
    max_lease_duration: Option<Duration>,

    retry_config: RetryConfig,

    /// Ignore NAKs.
    ignore_naks: bool,

    /// Server port config
    pub(crate) server_port: u16,

    /// Client port config
    pub(crate) client_port: u16,

    /// A buffer contains options additional to be added to outgoing DHCP
    /// packets.
    outgoing_options: &'a [DhcpOption<'a>],
    /// A buffer containing all requested parameters.
    parameter_request_list: Option<&'a [u8]>,

    /// Incoming DHCP packets are copied into this buffer, overwriting the previous.
    receive_packet_buffer: Option<&'a mut [u8]>,

    /// Waker registration
    #[cfg(feature = "async")]
    waker: WakerRegistration,
}

/// DHCP client socket.
///
/// The socket acquires an IP address configuration through DHCP autonomously.
/// You must query the configuration with `.poll()` after every call to `Interface::poll()`,
/// and apply the configuration to the `Interface`.
// 196
impl<'a> Socket<'a> {
    /// Create a DHCPv4 socket
    #[allow(clippy::new_without_default)]
    // 199
    pub fn new() -> Self {
        Socket {
            state: ClientState::Discovering(DiscoverState {
                retry_at: Instant::from_millis(0),
            }),
            config_changed: true,
            transaction_id: 1,
            max_lease_duration: None,
            retry_config: RetryConfig::default(),
            ignore_naks: false,
            outgoing_options: &[],
            parameter_request_list: None,
            receive_packet_buffer: None,
            #[cfg(feature = "async")]
            waker: WakerRegistration::new(),
            server_port: DHCP_SERVER_PORT,
            client_port: DHCP_CLIENT_PORT,
        }
    }

    /// Set the retry/timeouts configuration.
    // 220
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }

    /// Set the outgoing options.
    // 230
    pub fn set_outgoing_options(&mut self, options: &'a [DhcpOption<'a>]) {
        self.outgoing_options = options;
    }

    /// Set the max lease duration.
    ///
    /// When set, the lease duration will be capped at the configured duration if the
    /// DHCP server gives us a longer lease. This is generally not recommended, but
    /// can be useful for debugging or reacting faster to network configuration changes.
    ///
    /// If None, no max is applied (the lease duration from the DHCP server is used.)
    // 261
    pub fn set_max_lease_duration(&mut self, max_lease_duration: Option<Duration>) {
        self.max_lease_duration = max_lease_duration;
    }

    /// Set whether to ignore NAKs.
    ///
    /// This is not compliant with the DHCP RFCs, since theoretically
    /// we must stop using the assigned IP when receiving a NAK. This
    /// can increase reliability on broken networks with buggy routers
    /// or rogue DHCP servers, however.
    // 278
    pub fn set_ignore_naks(&mut self, ignore_naks: bool) {
        self.ignore_naks = ignore_naks;
    }

    /// Set the server/client port
    ///
    /// Allows you to specify the ports used by DHCP.
    /// This is meant to support esoteric usecases allowed by the dhclient program.
    pub fn set_ports(&mut self, server_port: u16, client_port: u16) {
        self.server_port = server_port;
        self.client_port = client_port;
    }

    /// Reset state and restart discovery phase.
    ///
    /// Use this to speed up acquisition of an address in a new
    /// network if a link was down and it is now back up.
    // 731
    pub fn reset(&mut self) {
        net_trace!("DHCP reset");
        if let ClientState::Renewing(_) = &self.state {
            self.config_changed();
        }
        self.state = ClientState::Discovering(DiscoverState {
            retry_at: Instant::from_millis(0),
        });
    }

    /// This function _must_ be called when the configuration provided to the
    /// interface, by this DHCP socket, changes. It will update the `config_changed` field
    /// so that a subsequent call to `poll` will yield an event, and wake a possible waker.
    // 769
    pub(crate) fn config_changed(&mut self) {
        self.config_changed = true;
        #[cfg(feature = "async")]
        self.waker.wake();
    }
}
