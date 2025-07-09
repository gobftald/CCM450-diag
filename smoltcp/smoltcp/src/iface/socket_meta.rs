// 1
use super::SocketHandle;

// 2
use crate::{time::Instant, wire::IpAddress};

/// Neighbor dependency.
///
/// This enum tracks whether the socket should be polled based on the neighbor
/// it is going to send packets to.
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 14
enum NeighborState {
    /// Socket can be polled immediately.
    #[default]
    Active,
    /// Socket should not be polled until either `silent_until` passes or
    /// `neighbor` appears in the neighbor cache.
    Waiting {
        neighbor: IpAddress,
        silent_until: Instant,
    },
}

/// Network socket metadata.
///
/// This includes things that only external (to the socket, that is) code
/// is interested in, but which are more conveniently stored inside the socket
/// itself.
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 33
pub(crate) struct Meta {
    /// Handle of this socket within its enclosing `SocketSet`.
    /// Mainly useful for debug output.
    pub(crate) handle: SocketHandle,
    /// See [NeighborState](struct.NeighborState.html).
    neighbor_state: NeighborState,
}
