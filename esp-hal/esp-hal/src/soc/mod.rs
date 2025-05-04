// 5
pub use self::implementation::*;

// 9
#[cfg_attr(esp32c3, path = "esp32c3/mod.rs")]
mod implementation;
