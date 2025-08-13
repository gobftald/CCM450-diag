/*! Network interface logic.

The `iface` module deals with the *network interfaces*. It filters incoming frames,
provides lookup and caching of hardware addresses, and handles management packets.
*/

// 8
mod interface;

#[cfg(feature = "medium-ethernet")]
// 10
mod neighbor;
mod route;

// 14
mod socket_meta;
mod socket_set;

// 17
mod packet;

// 21
pub use self::interface::{Config, Interface, InterfaceInner as Context};

// 25
pub use self::route::Routes;
pub use self::socket_set::{SocketHandle, SocketSet, SocketStorage};
