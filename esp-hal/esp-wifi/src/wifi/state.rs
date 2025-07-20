// 1
use core::sync::atomic::Ordering;

// 5
use super::WifiEvent;

/// Wifi interface state
#[portable_atomic_enum::atomic_enum]
#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WifiState {
    StaStarted,
    StaConnected,
    StaDisconnected,
    StaStopped,

    ApStarted,
    ApStopped,

    Invalid,
}

// 23
impl From<WifiEvent> for WifiState {
    fn from(event: WifiEvent) -> WifiState {
        match event {
            WifiEvent::StaStart => WifiState::StaStarted,
            WifiEvent::StaConnected => WifiState::StaConnected,
            WifiEvent::StaDisconnected => WifiState::StaDisconnected,
            WifiEvent::StaStop => WifiState::StaStopped,
            WifiEvent::ApStart => WifiState::ApStarted,
            WifiEvent::ApStop => WifiState::ApStopped,
            _ => WifiState::Invalid,
        }
    }
}

// 37
pub(crate) static STA_STATE: AtomicWifiState = AtomicWifiState::new(WifiState::Invalid);
pub(crate) static AP_STATE: AtomicWifiState = AtomicWifiState::new(WifiState::Invalid);

/// Get the current state of the AP
// 41
pub fn ap_state() -> WifiState {
    AP_STATE.load(Ordering::Relaxed)
}

// 50
pub(crate) fn update_state(event: WifiEvent, handled: bool) {
    match event {
        WifiEvent::StaConnected
        | WifiEvent::StaDisconnected
        | WifiEvent::StaStart
        | WifiEvent::StaStop => STA_STATE.store(WifiState::from(event), Ordering::Relaxed),

        WifiEvent::ApStart | WifiEvent::ApStop => {
            AP_STATE.store(WifiState::from(event), Ordering::Relaxed)
        }

        other => {
            if !handled {
                debug!("Unhandled event: {:?}", other)
            }
        }
    }
}
