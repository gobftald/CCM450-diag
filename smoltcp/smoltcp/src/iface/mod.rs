/*! Network interface logic.

The `iface` module deals with the *network interfaces*. It filters incoming frames,
provides lookup and caching of hardware addresses, and handles management packets.
*/

// 7
mod fragmentation;

// 8
mod interface;

#[cfg(feature = "medium-ethernet")]
// 10
mod neighbor;
mod route;

// 14
mod socket_meta;
mod socket_set;

// 21
pub use self::interface::{Config, Interface};

// 25
pub use self::route::Routes;
pub use self::socket_set::{SocketHandle, SocketSet, SocketStorage};
