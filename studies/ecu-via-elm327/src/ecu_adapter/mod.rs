#[cfg_attr(feature = "elm327", path = "elm327.rs")]
mod adapter_implementation;
pub use adapter_implementation::Adapter;

use esp_hal::uart::{RxError, TxError};

pub trait Adapters {
    async fn write_async(&mut self, request: &[u8]) -> Result<usize, TxError>;
    async fn read_async(&mut self, response: &mut [u8]) -> Result<usize, RxError>;
}
