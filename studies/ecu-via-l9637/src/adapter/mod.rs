#[cfg_attr(feature = "elm327", path = "elm327.rs")]
#[cfg_attr(feature = "l9637", path = "l9637.rs")]
mod adapter_impl;
pub use adapter_impl::{Adapter, AdapterError};

pub trait Adapters {
    async fn write(&mut self, request: &[u8]) -> Result<usize, AdapterError>;
    async fn read(&mut self, response: &mut [u8]) -> Result<usize, AdapterError>;
}
