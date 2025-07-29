use allocator_api2::vec::Vec;

// 36
use super::fragmentation::{Fragmenter, FragmentsBuffer};

// 39
use super::neighbor::Cache as NeighborCache;

// 41
use crate::config::IFACE_MAX_ADDR_COUNT;
use crate::iface::Routes;

// 44
use crate::phy::{Device, DeviceCapabilities};
use crate::rand::Rand;

// 47
use crate::time::Instant;

// 49
use crate::wire::{HardwareAddress, IpCidr};

/// A  network interface.
///
/// The network interface logically owns a number of other data structures; to avoid
/// a dependency on heap allocation, it instead owns a `BorrowMut<[T]>`, which can be
/// a `&mut [T]`, or `Vec<T>` if a heap is available.
#[allow(unused)]
// 111
pub struct Interface {
    pub(crate) inner: InterfaceInner,
    fragments: FragmentsBuffer,
    fragmenter: Fragmenter,
}

/// The device independent part of an Ethernet network interface.
///
/// Separating the device from the data required for processing and dispatching makes
/// it possible to borrow them independently. For example, the tx and rx tokens borrow
/// the `device` mutably until they're used, which makes it impossible to call other
/// methods on the `Interface` in this time (since its `device` field is borrowed
/// exclusively). However, it is still possible to call methods on its `inner` field.
// 124
pub struct InterfaceInner {
    caps: DeviceCapabilities,
    now: Instant,
    rand: Rand,

    #[cfg(feature = "medium-ethernet")]
    neighbor_cache: NeighborCache,
    hardware_addr: HardwareAddress,
    /*
    #[cfg(feature = "medium-ieee802154")]
    sequence_no: u8,
    #[cfg(feature = "medium-ieee802154")]
    pan_id: Option<Ieee802154Pan>,
    #[cfg(feature = "proto-ipv4-fragmentation")]
    ipv4_id: u16,
    #[cfg(feature = "proto-sixlowpan")]
    sixlowpan_address_context:
        Vec<SixlowpanAddressContext, IFACE_MAX_SIXLOWPAN_ADDRESS_CONTEXT_COUNT>,
    #[cfg(feature = "proto-sixlowpan-fragmentation")]
    tag: u16,
    */
    //ip_addrs: Vec<IpCidr, IFACE_MAX_ADDR_COUNT>,
    ip_addrs: Vec<IpCidr>,
    any_ip: bool,
    routes: Routes,
    #[cfg(feature = "multicast")]
    multicast: multicast::State,
}

/// Configuration structure used for creating a network interface.
#[non_exhaustive]
// 152
pub struct Config {
    /// Random seed.
    ///
    /// It is strongly recommended that the random seed is different on each boot,
    /// to avoid problems with TCP port/sequence collisions.
    ///
    /// The seed doesn't have to be cryptographically secure.
    pub random_seed: u64,

    /// Set the Hardware address the interface will use.
    ///
    /// # Panics
    /// Creating the interface panics if the address is not unicast.
    pub hardware_addr: HardwareAddress,
    /*
    /// Set the IEEE802.15.4 PAN ID the interface will use.
    ///
    /// **NOTE**: we use the same PAN ID for destination and source.
    #[cfg(feature = "medium-ieee802154")]
    pub pan_id: Option<Ieee802154Pan>,
    */
}

// 174
impl Config {
    // 175
    pub fn new(hardware_addr: HardwareAddress) -> Self {
        Config {
            random_seed: 0,
            hardware_addr,
            /*
            #[cfg(feature = "medium-ieee802154")]
            pan_id: None,
            */
        }
    }
}

// 185
impl Interface {
    /// Create a network interface using the previously provided configuration.
    ///
    /// # Panics
    /// This function panics if the [`Config::hardware_address`] does not match
    /// the medium of the device.
    // 191
    pub fn new(config: Config, device: &mut (impl Device + ?Sized), now: Instant) -> Self {
        let caps = device.capabilities();
        assert_eq!(
            config.hardware_addr.medium(),
            caps.medium,
            "The hardware address does not match the medium of the interface."
        );

        let mut rand = Rand::new(config.random_seed);

        #[cfg(feature = "proto-ipv4")]
        let mut ipv4_id;

        #[cfg(feature = "proto-ipv4")]
        loop {
            ipv4_id = rand.rand_u16();
            if ipv4_id != 0 {
                break;
            }
        }

        Interface {
            fragments: FragmentsBuffer {},
            fragmenter: Fragmenter::new(),
            inner: InterfaceInner {
                now,
                caps,
                hardware_addr: config.hardware_addr,
                //ip_addrs: Vec::new(),
                ip_addrs: Vec::with_capacity(IFACE_MAX_ADDR_COUNT),
                any_ip: false,
                routes: Routes::new(),
                #[cfg(feature = "medium-ethernet")]
                neighbor_cache: NeighborCache::new(),
                #[cfg(feature = "multicast")]
                multicast: multicast::State::new(),
                rand,
            },
        }
    }

    /// Update the IP addresses of the interface.
    ///
    /// # Panics
    /// This function panics if any of the addresses are not unicast.
    // 361
    //pub fn update_ip_addrs<F: FnOnce(&mut Vec<IpCidr, IFACE_MAX_ADDR_COUNT>)>(&mut self, f: F) {
    pub fn update_ip_addrs<F: FnOnce(&mut Vec<IpCidr>)>(&mut self, f: F) {
        f(&mut self.inner.ip_addrs);
        InterfaceInner::flush_neighbor_cache(&mut self.inner);
        InterfaceInner::check_ip_addrs(&self.inner.ip_addrs);
    }

    // 385
    pub fn routes_mut(&mut self) -> &mut Routes {
        &mut self.inner.routes
    }
}

// 767
impl InterfaceInner {
    // 817
    fn check_ip_addrs(addrs: &[IpCidr]) {
        for cidr in addrs {
            if !cidr.address().is_unicast() && !cidr.address().is_unspecified() {
                panic!("IP address {} is not unicast", cidr.address())
            }
        }
    }

    // 1123
    fn flush_neighbor_cache(&mut self) {
        #[cfg(feature = "medium-ethernet")]
        self.neighbor_cache.flush()
    }
}
