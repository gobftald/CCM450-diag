// 4
//use heapless::LinearMap;
use allocator_api2::linear_map::LinearMap;

// 6
use crate::config::IFACE_NEIGHBOR_CACHE_COUNT;
use crate::time::Instant;
use crate::wire::{HardwareAddress, IpAddress};

/// A cached neighbor.
///
/// A neighbor mapping translates from a protocol address to a hardware address,
/// and contains the timestamp past which the mapping should be discarded.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 16
pub struct Neighbor {
    hardware_addr: HardwareAddress,
    expires_at: Instant,
}

/// A neighbor cache backed by a map.
#[derive(Debug)]
// 46
pub struct Cache {
    storage: LinearMap<IpAddress, Neighbor, IFACE_NEIGHBOR_CACHE_COUNT>,
    silent_until: Instant,
}

// 51
impl Cache {
    /// Create a cache.
    // 59
    pub fn new() -> Self {
        Self {
            storage: LinearMap::new(),
            silent_until: Instant::from_millis(0),
        }
    }

    // 174
    pub(crate) fn flush(&mut self) {
        self.storage.clear()
    }
}
