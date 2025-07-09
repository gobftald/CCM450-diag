#![no_std]

#[macro_use(unwrap, debug, info)]
extern crate console;

// 17
mod driver_util;

// 24
mod time;

// 28
use core::cell::RefCell;

// 30
use core::mem::MaybeUninit;

// 34
pub use embassy_net_driver as driver;
use embassy_net_driver::Driver;
use embassy_sync::waitqueue::WakerRegistration;

// 37
use embassy_time::Instant;
//use heapless::Vec;
use allocator_api2::vec::Vec;

#[cfg(any(feature = "dns", feature = "dhcpv4"))]
// 44
use smoltcp::iface::SocketHandle;

// 45
use smoltcp::iface::{Interface, SocketSet, SocketStorage};
use smoltcp::phy::Medium;

// 47
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4::{self, RetryConfig};

#[cfg(feature = "medium-ethernet")]
// 50
pub use smoltcp::wire::EthernetAddress;

#[cfg(feature = "medium-ethernet")]
// 52
pub use smoltcp::wire::HardwareAddress;

// 57
pub use smoltcp::wire::IpCidr;

#[cfg(feature = "proto-ipv4")]
// 59
pub use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

// 63
use crate::driver_util::DriverAdapter;
use crate::time::instant_to_smoltcp;

// 66
const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;

/// Memory resources needed for a network stack.
// 74
pub struct StackResources<const SOCK: usize> {
    sockets: MaybeUninit<[SocketStorage<'static>; SOCK]>,
    inner: MaybeUninit<RefCell<Inner>>,
    #[cfg(feature = "dns")]
    queries: MaybeUninit<[Option<dns::DnsQuery>; MAX_QUERIES]>,
    #[cfg(feature = "dhcpv4-hostname")]
    hostname: HostnameResources,
}

// 89
impl<const SOCK: usize> StackResources<SOCK> {
    /// Create a new set of stack resources.
    pub const fn new() -> Self {
        Self {
            sockets: MaybeUninit::uninit(),
            inner: MaybeUninit::uninit(),
            #[cfg(feature = "dns")]
            queries: MaybeUninit::uninit(),
            #[cfg(feature = "dhcpv4-hostname")]
            hostname: HostnameResources {
                option: MaybeUninit::uninit(),
                data: MaybeUninit::uninit(),
            },
        }
    }
}

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
// 134
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

#[cfg(feature = "dhcpv4")]
// 156
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

/// Network stack runner.
///
/// You must call [`Runner::run()`] in a background task for the network stack to work.
// 248
pub struct Runner<'d, D: Driver> {
    driver: D,
    stack: Stack<'d>,
}

/// Network stack handle
///
/// Use this to create sockets. It's `Copy`, so you can pass
/// it by value instead of by reference.
#[derive(Copy, Clone)]
// 258
pub struct Stack<'d> {
    inner: &'d RefCell<Inner>,
}

// 262
pub(crate) struct Inner {
    pub(crate) sockets: SocketSet<'static>, // Lifetime type-erased.
    pub(crate) iface: Interface,
    /// Waker used for triggering polls.
    pub(crate) waker: WakerRegistration,
    /// Waker used for waiting for link up or config up.
    state_waker: WakerRegistration,
    hardware_address: HardwareAddress,
    next_local_port: u16,
    link_up: bool,
    #[cfg(feature = "proto-ipv4")]
    static_v4: Option<StaticConfigV4>,
    #[cfg(feature = "proto-ipv6")]
    static_v6: Option<StaticConfigV6>,
    #[cfg(feature = "dhcpv4")]
    dhcp_socket: Option<SocketHandle>,
    #[cfg(feature = "dns")]
    dns_socket: SocketHandle,
    #[cfg(feature = "dns")]
    dns_waker: WakerRegistration,
    #[cfg(feature = "dhcpv4-hostname")]
    hostname: *mut HostnameResources,
}

/// Create a new network stack.
// 290
pub fn new<'d, D: Driver, const SOCK: usize>(
    mut driver: D,
    config: Config,
    resources: &'d mut StackResources<SOCK>,
    random_seed: u64,
) -> (Stack<'d>, Runner<'d, D>) {
    let (hardware_address, medium) = to_smoltcp_hardware_address(driver.hardware_address());
    let mut iface_cfg = smoltcp::iface::Config::new(hardware_address);
    iface_cfg.random_seed = random_seed;

    let iface = Interface::new(
        iface_cfg,
        &mut DriverAdapter {
            inner: &mut driver,
            cx: None,
            medium,
        },
        instant_to_smoltcp(Instant::now()),
    );

    unsafe fn transmute_slice<T>(x: &mut [T]) -> &'static mut [T] {
        core::mem::transmute(x)
    }
    let sockets = resources.sockets.write([SocketStorage::EMPTY; SOCK]);
    #[allow(unused_mut)]
    let mut sockets: SocketSet<'static> = SocketSet::new(unsafe { transmute_slice(sockets) });

    let next_local_port =
        (random_seed % (LOCAL_PORT_MAX - LOCAL_PORT_MIN) as u64) as u16 + LOCAL_PORT_MIN;

    #[cfg(feature = "dns")]
    let dns_socket = sockets.add(dns::Socket::new * ());

    let mut inner = Inner {
        sockets,
        iface,
        waker: WakerRegistration::new(),
        state_waker: WakerRegistration::new(),
        next_local_port,
        hardware_address,
        link_up: false,
        #[cfg(feature = "proto-ipv4")]
        static_v4: None,
        #[cfg(feature = "proto-ipv6")]
        static_v6: None,
        #[cfg(feature = "dhcpv4")]
        dhcp_socket: None,
        #[cfg(feature = "dns")]
        dns_socket,
        #[cfg(feature = "dns")]
        dns_waker: WakerRegistration::new(),
        #[cfg(feature = "dhcpv4-hostname")]
        hostname: &mut resources.hostname,
    };

    #[cfg(feature = "proto-ipv4")]
    inner.set_config_v4(config.ipv4);
    #[cfg(feature = "proto-ipv6")]
    inner.set_config_v6(config.ipv6);
    inner.apply_static_config();

    let inner = &*resources.inner.write(RefCell::new(inner));
    let stack = Stack { inner };
    (stack, Runner { driver, stack })
}

// 363
fn to_smoltcp_hardware_address(addr: driver::HardwareAddress) -> (HardwareAddress, Medium) {
    match addr {
        #[cfg(feature = "medium-ethernet")]
        driver::HardwareAddress::Ethernet(eth) => (HardwareAddress::Ethernet(EthernetAddress(eth)), Medium::Ethernet),
        /*
        #[cfg(feature = "medium-ieee802154")]
        driver::HardwareAddress::Ieee802154(ieee) => (
            HardwareAddress::Ieee802154(Ieee802154Address::Extended(ieee)),
            Medium::Ieee802154,
        ),
        #[cfg(feature = "medium-ip")]
        driver::HardwareAddress::Ip => (HardwareAddress::Ip, Medium::Ip),
        */

        #[allow(unreachable_patterns)]
        _ => panic!(
            "Unsupported medium {:?}. Make sure to enable the right medium feature in embassy-net's Cargo features.",
            addr
        ),
    }
}

// 646
impl Inner {
    #[cfg(feature = "proto-ipv4")]
    // 655
    pub fn set_config_v4(&mut self, config: ConfigV4) {
        // Handle static config.
        self.static_v4 = match config.clone() {
            ConfigV4::None => None,
            #[cfg(feature = "dhcpv4")]
            ConfigV4::Dhcp(_) => None,
            ConfigV4::Static(c) => Some(c),
        };

        // Handle DHCP config.
        #[cfg(feature = "dhcpv4")]
        match config {
            ConfigV4::Dhcp(c) => {
                // Create the socket if it doesn't exist.
                if self.dhcp_socket.is_none() {
                    let socket = smoltcp::socket::dhcpv4::Socket::new();
                    let handle = self.sockets.add(socket);
                    self.dhcp_socket = Some(handle);
                }

                // Configure it
                let socket = self
                    .sockets
                    .get_mut::<dhcpv4::Socket>(unwrap!(self.dhcp_socket));
                socket.set_ignore_naks(c.ignore_naks);
                socket.set_max_lease_duration(
                    c.max_lease_duration.map(crate::time::duration_to_smoltcp),
                );
                socket.set_ports(c.server_port, c.client_port);
                socket.set_retry_config(c.retry_config);
                socket.set_outgoing_options(&[]);

                socket.set_outgoing_options(&[]);
                #[cfg(feature = "dhcpv4-hostname")]
                if let Some(h) = c.hostname {}

                socket.reset();
            }
            _ => {
                // Remove DHCP socket if any.
                if let Some(socket) = self.dhcp_socket {
                    self.sockets.remove(socket);
                    self.dhcp_socket = None;
                }
            }
        }
    }

    // 721
    fn apply_static_config(&mut self) {
        //let mut addrs = Vec::new();
        let mut addrs = Vec::<IpCidr>::with_capacity(smoltcp::config::IFACE_MAX_ADDR_COUNT);
        #[cfg(feature = "dns")]
        let mut dns_servers: Vec<_, 6> = Vec::new();
        #[cfg(feature = "proto-ipv4")]
        let mut gateway_v4 = None;
        #[cfg(feature = "proto-ipv6")]
        let mut gateway_v6 = None;

        #[cfg(feature = "proto-ipv4")]
        if let Some(config) = &self.static_v4 {
            debug!("IPv4: UP");
            debug!("   IP address:      {:?}", config.address);
            debug!("   Default gateway: {:?}", config.gateway);

            //unwrap!(addrs.push(IpCidr::Ipv4(config.address)).ok());
            unwrap!(addrs.push_within_capacity(IpCidr::Ipv4(config.address)));
            gateway_v4 = config.gateway;
            #[cfg(feature = "dns")]
            for s in &config.dns_servers {
                debug!("   DNS server:      {:?}", s);
                unwrap!(dns_servers.push(s.clone().into()).ok());
            }
        } else {
            info!("IPv4: DOWN");
        }

        #[cfg(feature = "proto-ipv6")]
        if let Some(config) = &self.static_v6 {}

        // Apply addresses
        self.iface.update_ip_addrs(|a| *a = addrs);

        // Apply gateways
        #[cfg(feature = "proto-ipv4")]
        if let Some(gateway) = gateway_v4 {
            unwrap!(self.iface.routes_mut().add_default_ipv4_route(gateway));
        } else {
            self.iface.routes_mut().remove_default_ipv4_route();
        }
        #[cfg(feature = "proto-ipv6")]
        if let Some(gateway) = gateway_v6 {}

        // Apply DNS servers
        #[cfg(feature = "dns")]
        if !dns_servers.is_empty() {}

        self.state_waker.wake();
    }
}
