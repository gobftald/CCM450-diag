#[cfg_attr(feature = "elm327", path = "elm327.rs")]
mod adapter_implementation;
pub use adapter_implementation::Adapter;

use esp_hal::uart::{RxError, TxError};

pub trait Adapters {
    async fn connect(&mut self, secret_key: u32) -> Result<(), AdapterError>;
    async fn write(&mut self, request: &[u8]) -> Result<usize, AdapterError>;
    async fn read(&mut self, response: &mut [u8]) -> Result<usize, AdapterError>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdapterError {
    Tx(TxError),
    Rx(RxError),
    Timeout,
    Elm327Nok,
    InitError,
    AuthError,
}
