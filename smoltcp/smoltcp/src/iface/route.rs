// 1
//use heapless::Vec;
use allocator_api2::vec::Vec;

// 3
use crate::config::IFACE_MAX_ROUTE_COUNT;
use crate::time::Instant;
use crate::wire::{IpAddress, IpCidr};
#[cfg(feature = "proto-ipv4")]
use crate::wire::{Ipv4Address, Ipv4Cidr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 13
pub struct RouteTableFull;

/// A prefix of addresses that should be routed via a router
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 27
pub struct Route {
    pub cidr: IpCidr,
    pub via_router: IpAddress,
    /// `None` means "forever".
    pub preferred_until: Option<Instant>,
    /// `None` means "forever".
    pub expires_at: Option<Instant>,
}

#[cfg(feature = "proto-ipv4")]
// 37
const IPV4_DEFAULT: IpCidr = IpCidr::Ipv4(Ipv4Cidr::new(Ipv4Address::new(0, 0, 0, 0), 0));

// 42
impl Route {
    /// Returns a route to 0.0.0.0/0 via the `gateway`, with no expiry.
    #[cfg(feature = "proto-ipv4")]
    pub fn new_ipv4_gateway(gateway: Ipv4Address) -> Route {
        Route {
            cidr: IPV4_DEFAULT,
            via_router: gateway.into(),
            preferred_until: None,
            expires_at: None,
        }
    }
}

/// A routing table.
#[derive(Debug)]
// 68
pub struct Routes {
    //storage: Vec<Route, IFACE_MAX_ROUTE_COUNT>,
    storage: Vec<Route>,
}

// 72
impl Routes {
    /// Creates a new empty routing table.
    // 74
    pub fn new() -> Self {
        Self {
            //storage: Vec::new(),
            storage: Vec::with_capacity(IFACE_MAX_ROUTE_COUNT),
        }
    }

    /// Add a default ipv4 gateway (ie. "ip route add 0.0.0.0/0 via `gateway`").
    ///
    /// On success, returns the previous default route, if any.
    #[cfg(feature = "proto-ipv4")]
    // 89
    pub fn add_default_ipv4_route(
        &mut self,
        gateway: Ipv4Address,
    ) -> Result<Option<Route>, RouteTableFull> {
        let old = self.remove_default_ipv4_route();
        self.storage
            .push_within_capacity(Route::new_ipv4_gateway(gateway))
            .map_err(|_| RouteTableFull)?;
        Ok(old)
    }

    /// Remove the default ipv4 gateway
    ///
    /// On success, returns the previous default route, if any.
    #[cfg(feature = "proto-ipv4")]
    // 119
    pub fn remove_default_ipv4_route(&mut self) -> Option<Route> {
        if let Some((i, _)) = self
            .storage
            .iter()
            .enumerate()
            .find(|(_, r)| r.cidr == IPV4_DEFAULT)
        {
            //Some(self.storage.remove(i))
            Some(self.storage.swap_remove(i))
        } else {
            None
        }
    }

    // 149
    pub(crate) fn lookup(&self, addr: &IpAddress, timestamp: Instant) -> Option<IpAddress> {
        assert!(addr.is_unicast());

        self.storage
            .iter()
            // Keep only matching routes
            .filter(|route| {
                if let Some(expires_at) = route.expires_at {
                    if timestamp > expires_at {
                        return false;
                    }
                }
                route.cidr.contains_addr(addr)
            })
            // pick the most specific one (highest prefix_len)
            .max_by_key(|route| route.cidr.prefix_len())
            .map(|route| route.via_router)
    }
}
