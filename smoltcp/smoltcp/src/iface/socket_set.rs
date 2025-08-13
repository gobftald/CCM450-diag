// 2
use managed::ManagedSlice;

// 4
use super::socket_meta::Meta;
use crate::socket::{AnySocket, Socket};

/// Opaque struct with space for storing one socket.
///
/// This is public so you can use it to allocate space for storing
/// sockets when creating an Interface.
#[derive(Debug, Default)]
// 12
pub struct SocketStorage<'a> {
    inner: Option<Item<'a>>,
}

// 16
impl<'a> SocketStorage<'a> {
    pub const EMPTY: Self = Self { inner: None };
}

/// An item of a socket set.
#[derive(Debug)]
// 22
pub(crate) struct Item<'a> {
    pub(crate) meta: Meta,
    pub(crate) socket: Socket<'a>,
}

/// A handle, identifying a socket in an Interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 30
pub struct SocketHandle(usize);

/// An extensible set of sockets.
///
/// The lifetime `'a` is used when storing a `Socket<'a>`.  If you're using
/// owned buffers for your sockets (passed in as `Vec`s) you can use
/// `SocketSet<'static>`.
#[derive(Debug)]
// 44
pub struct SocketSet<'a> {
    sockets: ManagedSlice<'a, SocketStorage<'a>>,
}

// 48
impl<'a> SocketSet<'a> {
    /// Create a socket set using the provided storage.
    // 50
    pub fn new<SocketsT>(sockets: SocketsT) -> SocketSet<'a>
    where
        SocketsT: Into<ManagedSlice<'a, SocketStorage<'a>>>,
    {
        let sockets = sockets.into();
        SocketSet { sockets }
    }

    /// Add a socket to the set, and return its handle.
    ///
    /// # Panics
    /// This function panics if the storage is fixed-size (not a `Vec`) and is full.
    // 62
    pub fn add<T: AnySocket<'a>>(&mut self, socket: T) -> SocketHandle {
        fn put<'a>(index: usize, slot: &mut SocketStorage<'a>, socket: Socket<'a>) -> SocketHandle {
            net_trace!("[{}]: adding", index);
            let handle = SocketHandle(index);
            let mut meta = Meta::default();
            meta.handle = handle;
            *slot = SocketStorage {
                inner: Some(Item { meta, socket }),
            };
            handle
        }

        let socket = socket.upcast();

        for (index, slot) in self.sockets.iter_mut().enumerate() {
            if slot.inner.is_none() {
                return put(index, slot, socket);
            }
        }

        match &mut self.sockets {
            ManagedSlice::Borrowed(_) => panic!("adding a socket to a full SocketSet"),
            #[cfg(feature = "alloc")]
            ManagedSlice::Owned(sockets) => {
                sockets.push(SocketStorage { inner: None });
                let index = sockets.len() - 1;
                put(index, &mut sockets[index], socket)
            }
        }
    }

    /// Get a mutable socket from the set by its handle, as mutable.
    ///
    /// # Panics
    /// This function may panic if the handle does not belong to this socket set
    /// or the socket has the wrong type.
    // 112
    pub fn get_mut<T: AnySocket<'a>>(&mut self, handle: SocketHandle) -> &mut T {
        match self.sockets[handle.0].inner.as_mut() {
            //Some(item) => T::downcast_mut(&mut item.socket)
            //    .expect("handle refers to a socket of a wrong type"),
            Some(item) => unwrap!(
                T::downcast_mut(&mut item.socket),
                "handle refers to a socket of a wrong type"
            ),
            None => panic!("handle does not refer to a valid socket"),
        }
    }

    /// Remove a socket from the set, without changing its state.
    ///
    /// # Panics
    /// This function may panic if the handle does not belong to this socket set.
    // 124
    pub fn remove(&mut self, handle: SocketHandle) -> Socket<'a> {
        net_trace!("[{}]: removing", handle.0);
        match self.sockets[handle.0].inner.take() {
            Some(item) => item.socket,
            None => panic!("handle does not refer to a valid socket"),
        }
    }

    /// Iterate every socket in this set.
    // 143
    pub(crate) fn items(&self) -> impl Iterator<Item = &Item<'a>> + '_ {
        self.sockets.iter().filter_map(|x| x.inner.as_ref())
    }

    /// Iterate every socket in this set.
    // 148
    pub(crate) fn items_mut(&mut self) -> impl Iterator<Item = &mut Item<'a>> + '_ {
        self.sockets.iter_mut().filter_map(|x| x.inner.as_mut())
    }
}
