#[cfg_attr(feature = "elm327", path = "elm327.rs")]
mod adapter_implementation;
pub use adapter_implementation::Adapter;

use esp_hal::uart::{RxError, TxError};

pub trait Adapters {
    async fn connect(&mut self) -> Result<(), TxError>;
    async fn write(&mut self, request: &[u8]) -> Result<usize, TxError>;

    // there is no conversion between results, so we can accept future directly
    fn read(&mut self, response: &mut [u8]) -> impl Future<Output = Result<usize, RxError>>;
}
