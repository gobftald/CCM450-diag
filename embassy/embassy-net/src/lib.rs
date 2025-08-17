#![no_std]

#[cfg(not(any(feature = "proto-ipv4", feature = "proto-ipv6")))]
compile_error!("You must enable at least one of the following features: proto-ipv4, proto-ipv6");

#[allow(unused_imports)]
#[macro_use(unwrap, debug, info, trace)]
extern crate console;

// 17
mod driver_util;

// 24
mod time;

#[cfg(feature = "udp")]
// 26
pub mod udp;

// 28
use core::cell::RefCell;
use core::future::{poll_fn, Future};

// 30
use core::mem::MaybeUninit;

// 31
use core::pin::pin;
use core::task::{Context, Poll};

// 34
pub use embassy_net_driver as driver;
use embassy_net_driver::{Driver, LinkState};
use embassy_sync::waitqueue::WakerRegistration;

// 37
use embassy_time::{Instant, Timer};
//use heapless::Vec;
use allocator_api2::vec::Vec;

//#[cfg(any(feature = "dns", feature = "dhcpv4"))]
#[cfg(feature = "dhcpv4")]
// 44
use smoltcp::iface::SocketHandle;
use smoltcp::iface::{Interface, SocketSet, SocketStorage};
use smoltcp::phy::Medium;
#[cfg(feature = "dhcpv4")]
use smoltcp::socket::dhcpv4::{self, RetryConfig};

#[cfg(feature = "medium-ethernet")]
// 50
pub use smoltcp::wire::EthernetAddress;
#[cfg(feature = "medium-ethernet")]
pub use smoltcp::wire::HardwareAddress;

// 57
pub use smoltcp::wire::IpCidr;
#[cfg(feature = "proto-ipv4")]
pub use smoltcp::wire::{Ipv4Address, Ipv4Cidr};

// 63
use crate::driver_util::DriverAdapter;
use crate::time::{instant_from_smoltcp, instant_to_smoltcp};

// 66
const LOCAL_PORT_MIN: u16 = 1025;
const LOCAL_PORT_MAX: u16 = 65535;

/// Memory resources needed for a network stack.
// 74
pub struct StackResources<const SOCK: usize> {
    sockets: MaybeUninit<[SocketStorage<'static>; SOCK]>,
    inner: MaybeUninit<RefCell<Inner>>,
    //#[cfg(feature = "dns")]
    //queries: MaybeUninit<[Option<dns::DnsQuery>; MAX_QUERIES]>,
    //#[cfg(feature = "dhcpv4-hostname")]
    //hostname: HostnameResources,
}

// 89
impl<const SOCK: usize> StackResources<SOCK> {
    /// Create a new set of stack resources.
    pub const fn new() -> Self {
        Self {
            sockets: MaybeUninit::uninit(),
            inner: MaybeUninit::uninit(),
            //#[cfg(feature = "dns")]
            //queries: MaybeUninit::uninit(),
            /*
            #[cfg(feature = "dhcpv4-hostname")]
            hostname: HostnameResources {
                option: MaybeUninit::uninit(),
                data: MaybeUninit::uninit(),
            },
            */
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
    /*
    /// Our hostname. This will be sent to the DHCP server as Option 12.
    #[cfg(feature = "dhcpv4-hostname")]
    pub hostname: Option<heapless::String<MAX_HOSTNAME_LEN>>,
    */
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
            //#[cfg(feature = "dhcpv4-hostname")]
            //hostname: None,
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

#[allow(dead_code)]
// we use hardware_address and next_local_port only in smoltcp
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
    //#[cfg(feature = "dns")]
    //dns_socket: SocketHandle,
    //#[cfg(feature = "dns")]
    //dns_waker: WakerRegistration,
    //#[cfg(feature = "dhcpv4-hostname")]
    //hostname: *mut HostnameResources,
}

/// Create a new network stack.
// 291
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
        //#[cfg(feature = "dns")]
        //dns_socket,
        //#[cfg(feature = "dns")]
        //dns_waker: WakerRegistration::new(),
        //#[cfg(feature = "dhcpv4-hostname")]
        //hostname: &mut resources.hostname,
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

// 362
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
            /*
            "Unsupported medium {:?}. Make sure to enable the right medium feature in embassy-net's Cargo features.",
            addr
            */
            "Unsupported medium. Make sure to enable the right medium feature in embassy-net's Cargo features."
        ),
    }
}

// 382
impl<'d> Stack<'d> {
    // 383
    fn with<R>(&self, f: impl FnOnce(&Inner) -> R) -> R {
        f(&self.inner.borrow())
    }

    //387
    fn with_mut<R>(&self, f: impl FnOnce(&mut Inner) -> R) -> R {
        f(&mut self.inner.borrow_mut())
    }

    /// Check whether the link is up.
    // 397
    pub fn is_link_up(&self) -> bool {
        self.with(|i| i.link_up)
    }

    /// Get the current IPv4 configuration.
    ///
    /// If using DHCP, this will be None if DHCP hasn't been able to
    /// acquire an IP address, or Some if it has.
    #[cfg(feature = "proto-ipv4")]
    // 499
    pub fn config_v4(&self) -> Option<StaticConfigV4> {
        self.with(|i| i.static_v4.clone())
    }
}

// 646
impl Inner {
    #[cfg(feature = "proto-ipv4")]
    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn get_local_port(&mut self) -> u16 {
        let res = self.next_local_port;
        self.next_local_port = if res >= LOCAL_PORT_MAX {
            LOCAL_PORT_MIN
        } else {
            res + 1
        };
        res
    }

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

        self.state_waker.wake();
    }

    // 798
    fn poll<D: Driver>(&mut self, cx: &mut Context<'_>, driver: &mut D) {
        self.waker.register(cx.waker());

        let (_hardware_addr, medium) = to_smoltcp_hardware_address(driver.hardware_address());

        self.iface.set_hardware_addr(_hardware_addr);

        let timestamp = instant_to_smoltcp(Instant::now());
        let mut smoldev = DriverAdapter {
            cx: Some(cx),
            inner: driver,
            medium,
        };
        self.iface.poll(timestamp, &mut smoldev, &mut self.sockets);

        // Update link up
        let old_link_up = self.link_up;
        self.link_up = driver.link_state(cx) == LinkState::Up;

        // Print when changed
        if old_link_up != self.link_up {
            info!("link_up = {:?}", self.link_up);
            self.state_waker.wake();
        }

        #[cfg(feature = "dhcpv4")]
        if let Some(dhcp_handle) = self.dhcp_socket {
            let socket = self.sockets.get_mut::<dhcpv4::Socket>(dhcp_handle);

            let configure = if self.link_up {
                if old_link_up != self.link_up {
                    socket.reset();
                }
                match socket.poll() {
                    None => false,
                    Some(dhcpv4::Event::Deconfigured) => {
                        self.static_v4 = None;
                        true
                    }
                    Some(dhcpv4::Event::Configured(config)) => {
                        self.static_v4 = Some(StaticConfigV4 {
                            address: config.address,
                            gateway: config.router,
                            dns_servers: config.dns_servers,
                        });
                        true
                    }
                }
            } else if old_link_up {
                socket.reset();
                self.static_v4 = None;
                true
            } else {
                false
            };
            if configure {
                self.apply_static_config()
            }
        }

        if let Some(poll_at) = self.iface.poll_at(timestamp, &mut self.sockets) {
            let t = pin!(Timer::at(instant_from_smoltcp(poll_at)));
            // embassy_time_driver::schedule_wake for cx
            // timer will wake Runner::run's await
            if t.poll(cx).is_ready() {
                // this wakes Runner immediately after we returned
                // and 'run' enter await (see below)
                cx.waker().wake_by_ref();
            }
        }
    }
}

// 880
impl<'d, D: Driver> Runner<'d, D> {
    /// Run the network stack.
    ///
    /// You must call this in a background task, to process network events.
    pub async fn run(&mut self) -> ! {
        poll_fn(|cx| {
            self.stack.with_mut(|i| i.poll(cx, &mut self.driver));
            Poll::<()>::Pending
        })
        // either poll_at timer or Stack.inner.waker wake
        .await;
        unreachable!()
    }
}
