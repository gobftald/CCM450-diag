// 1
use alloc::boxed::Box;

// 3
use esp_hal::sync::NonReentrantMutex;

// 5
use super::WifiEvent;

// 10
pub trait Event {
    /// Get the static reference to the handler for this event.
    fn handler() -> &'static NonReentrantMutex<Option<Box<Handler<Self>>>>;
    /// # Safety
    /// `ptr` must be a valid for casting to this event's inner event data.
    unsafe fn from_raw_event_data(ptr: *mut crate::binary::c_types::c_void) -> Self;
}

/// The type of handlers of events.
// 19
pub type Handler<T> = dyn FnMut(&T) + Sync + Send;

// 21
fn default_handler<Event: 'static>() -> Box<Handler<Event>> {
    fn drop_ref<T>(_: &T) {}
    // perf: `drop_ref` is a ZST [function item](https://doc.rust-lang.org/reference/types/function-item.html)
    // so this doesn't actually allocate.
    Box::new(drop_ref)
}

/// Extension trait for setting handlers for an event.
///
// 43
pub trait EventExt: Event + Sized + 'static {
    /// Get the handler for this event, replacing it with the default handler.
    // 45
    fn take_handler() -> Box<Handler<Self>> {
        Self::handler().with(|handler| handler.take().unwrap_or_else(default_handler::<Self>))
    }
    /// Set the handler for this event, returning the old handler.
    // 49
    fn replace_handler<F: FnMut(&Self) + Sync + Send + 'static>(f: F) -> Box<Handler<Self>> {
        Self::handler().with(|handler| {
            handler
                .replace(Box::new(f))
                .unwrap_or_else(default_handler::<Self>)
        })
    }
    /// Atomic combination of [`Self::take_handler`] and
    /// [`Self::replace_handler`]. Use this to add a new handler which runs
    /// after the previously registered handlers.
    // 59
    fn update_handler<F: FnMut(&Self) + Sync + Send + 'static>(mut f: F) {
        Self::handler().with(|handler| {
            let mut prev: Box<Handler<Self>> =
                handler.take().unwrap_or_else(default_handler::<Self>);
            handler.replace(Box::new(move |event| {
                prev(event);
                f(event)
            }));
        })
    }
}

// 70
impl<T: Event + 'static> EventExt for T {}

// 72
macro_rules! impl_wifi_event {
    // no data
    ($newtype:ident) => {
        /// See [`WifiEvent`].
        #[derive(Copy, Clone)]
        pub struct $newtype;
        impl Event for $newtype {
            unsafe fn from_raw_event_data(_: *mut crate::binary::c_types::c_void) -> Self {
                Self
            }
            fn handler() -> &'static NonReentrantMutex<Option<Box<Handler<Self>>>> {
                static HANDLE: NonReentrantMutex<Option<Box<Handler<$newtype>>>> =
                    NonReentrantMutex::new(None);
                &HANDLE
            }
        }
    };
    // data
    ($newtype:ident, $data:ident) => {
        pub use esp_wifi_sys_esp32c3::include::$data;
        /// See [`WifiEvent`].
        #[derive(Copy, Clone)]
        pub struct $newtype(pub $data);
        impl Event for $newtype {
            unsafe fn from_raw_event_data(ptr: *mut crate::binary::c_types::c_void) -> Self {
                Self(unsafe { *ptr.cast() })
            }
            fn handler() -> &'static NonReentrantMutex<Option<Box<Handler<Self>>>> {
                static HANDLE: NonReentrantMutex<Option<Box<Handler<$newtype>>>> =
                    NonReentrantMutex::new(None);
                &HANDLE
            }
        }
    };
}

// 106
impl_wifi_event!(WifiReady);
impl_wifi_event!(ScanDone, wifi_event_sta_scan_done_t);
impl_wifi_event!(StaStart);
impl_wifi_event!(StaStop);
impl_wifi_event!(StaConnected, wifi_event_sta_connected_t);
impl_wifi_event!(StaDisconnected, wifi_event_sta_disconnected_t);
impl_wifi_event!(StaAuthmodeChange, wifi_event_sta_authmode_change_t);
impl_wifi_event!(StaWpsErSuccess, wifi_event_sta_wps_er_success_t);
impl_wifi_event!(StaWpsErFailed);
impl_wifi_event!(StaWpsErTimeout);
impl_wifi_event!(StaWpsErPin, wifi_event_sta_wps_er_pin_t);
impl_wifi_event!(StaWpsErPbcOverlap);
impl_wifi_event!(ApStart);
impl_wifi_event!(ApStop);

/// Handle the given event using the registered event handlers.
// 165
pub fn handle<Event: EventExt>(event_data: &Event) -> bool {
    Event::handler().with(|handler| {
        if let Some(handler) = handler {
            handler(event_data);
            true
        } else {
            false
        }
    })
}

/// Handle an event given the raw pointers.
/// # Safety
/// The pointer should be valid to cast to `Event`'s inner type (if it has one)
// 179
pub(crate) unsafe fn handle_raw<Event: EventExt>(
    event_data: *mut crate::binary::c_types::c_void,
    event_data_size: usize,
) -> bool {
    debug_assert_eq!(
        event_data_size,
        core::mem::size_of::<Event>(),
        "wrong size event data"
    );

    let event = unsafe { Event::from_raw_event_data(event_data) };
    handle::<Event>(&event)
}

/// Handle event regardless of its type.
/// # Safety
/// Arguments should be self-consistent.
// 197
pub(crate) unsafe fn dispatch_event_handler(
    event: WifiEvent,
    event_data: *mut crate::binary::c_types::c_void,
    event_data_size: usize,
) -> bool {
    unsafe {
        match event {
            WifiEvent::WifiReady => handle_raw::<WifiReady>(event_data, event_data_size),
            WifiEvent::ScanDone => handle_raw::<ScanDone>(event_data, event_data_size),
            WifiEvent::StaStart => handle_raw::<StaStart>(event_data, event_data_size),
            WifiEvent::StaStop => handle_raw::<StaStop>(event_data, event_data_size),
            WifiEvent::StaConnected => handle_raw::<StaConnected>(event_data, event_data_size),
            WifiEvent::StaDisconnected => {
                handle_raw::<StaDisconnected>(event_data, event_data_size)
            }
            WifiEvent::StaAuthmodeChange => {
                handle_raw::<StaAuthmodeChange>(event_data, event_data_size)
            }
            WifiEvent::StaWpsErSuccess => {
                handle_raw::<StaWpsErSuccess>(event_data, event_data_size)
            }
            WifiEvent::StaWpsErFailed => handle_raw::<StaWpsErFailed>(event_data, event_data_size),
            WifiEvent::StaWpsErTimeout => {
                handle_raw::<StaWpsErTimeout>(event_data, event_data_size)
            }
            WifiEvent::StaWpsErPin => handle_raw::<StaWpsErPin>(event_data, event_data_size),
            WifiEvent::StaWpsErPbcOverlap => {
                handle_raw::<StaWpsErPbcOverlap>(event_data, event_data_size)
            }
            WifiEvent::ApStart => handle_raw::<ApStart>(event_data, event_data_size),
            WifiEvent::ApStop => handle_raw::<ApStop>(event_data, event_data_size),

            WifiEvent::ApStaconnected => false,
            WifiEvent::ApStadisconnected => false,
            WifiEvent::ApProbereqrecved => false,
            WifiEvent::FtmReport => false,
            WifiEvent::StaBssRssiLow => false,
            WifiEvent::ActionTxStatus => false,
            WifiEvent::RocDone => false,
            WifiEvent::StaBeaconTimeout => false,
            WifiEvent::ConnectionlessModuleWakeIntervalStart => false,
            WifiEvent::ApWpsRgSuccess => false,
            WifiEvent::ApWpsRgFailed => false,
            WifiEvent::ApWpsRgTimeout => false,
            WifiEvent::ApWpsRgPin => false,
            WifiEvent::ApWpsRgPbcOverlap => false,
            WifiEvent::ItwtSetup => false,
            WifiEvent::ItwtTeardown => false,
            WifiEvent::ItwtProbe => false,
            WifiEvent::ItwtSuspend => false,
            WifiEvent::TwtWakeup => false,
            WifiEvent::BtwtSetup => false,
            WifiEvent::BtwtTeardown => false,
            WifiEvent::NanStarted => false,
            WifiEvent::NanStopped => false,
            WifiEvent::NanSvcMatch => false,
            WifiEvent::NanReplied => false,
            WifiEvent::NanReceive => false,
            WifiEvent::NdpIndication => false,
            WifiEvent::NdpConfirm => false,
            WifiEvent::NdpTerminated => false,
            WifiEvent::HomeChannelChange => false,
            WifiEvent::StaNeighborRep => false,
        }
    }
}
