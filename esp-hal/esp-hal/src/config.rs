use crate::time::Duration;

/// Watchdog status.
#[derive(Default, PartialEq, Clone, Copy)]
pub enum WatchdogStatus {
    /// Enables a watchdog timer with the specified timeout.
    Enabled(Duration),
    /// Disables the watchdog timer.
    #[default]
    Disabled,
}

/// Watchdog configuration.
#[non_exhaustive]
#[derive(Default, Clone, Copy /*procmacros::BuilderLite*/)]
pub struct WatchdogConfig {
    #[cfg(not(any(esp32, esp32s2)))]
    /// Enable the super watchdog timer, which has a trigger time of slightly
    /// less than one second.
    swd: bool,
    /// Configures the reset watchdog timer.
    rwdt: WatchdogStatus,
    /// Configures the `timg0` watchdog timer.
    timg0: WatchdogStatus,
}
