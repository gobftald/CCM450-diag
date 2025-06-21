//! This module allows hooking `esp-wifi` into an external scheduler, instead
//! of using the integrated one as provided by the `preempt` module.

// 23
//pub trait Scheduler: Send + Sync + 'static {
pub trait Scheduler: 'static {
    /// This function is called by `esp-wifi` when starting up the WiFi stack.
    fn enable(&self);
}

// 59
unsafe extern "Rust" {
    fn esp_wifi_preempt_enable();
}

// 73
pub(crate) fn enable() {
    unsafe { esp_wifi_preempt_enable() }
}

/// Set the Scheduler implementation.
///
/// See the module documentation for an example.
#[macro_export]
// 109
macro_rules! scheduler_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[unsafe(no_mangle)]
        fn esp_wifi_preempt_enable() {
            <$t as $crate::preempt::Scheduler>::enable(&$name)
        }
    };
}
