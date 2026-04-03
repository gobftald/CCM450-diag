#[cfg_attr(feature = "elm327", path = "elm327.rs")]
mod adapter_impl;
pub use adapter_impl::{Adapter, AdapterError};
