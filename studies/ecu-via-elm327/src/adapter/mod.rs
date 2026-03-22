#[cfg_attr(feature = "elm327", path = "elm327.rs")]
mod adapter_impl;
pub use adapter_impl::Adapter;

mod utils;

use esp_hal::uart::{RxError, TxError};
#[allow(unused)]
pub trait Adapters {
    async fn connect(&mut self, secret_key: u32) -> Result<(), AdapterError>;

    async fn read_data_by_common_id(
        &mut self,
        ids: &[u8],
        reply: &mut [u8],
    ) -> Result<usize, AdapterError>;

    async fn read_diagnostic_trouble_codes_by_status(
        &mut self,
        reply: &mut [u8],
    ) -> Result<usize, AdapterError>;

    async fn clear_diagnostic_information(&mut self) -> Result<(), AdapterError>;

    async fn raw_request(
        &mut self,
        request: &[u8],
        reply: &mut [u8],
    ) -> Result<usize, AdapterError>;

    async fn write(&mut self, request: &[u8]) -> Result<usize, AdapterError>;
    async fn read(&mut self, response: &mut [u8]) -> Result<usize, AdapterError>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdapterError {
    Tx(TxError),
    Rx(RxError),
    Timeout,
    Elm327Nok,
    InitError,
    AuthError,
    ParseIntError,
    EcuSpecificError(u8),
}

impl From<AdapterError> for u8 {
    fn from(value: AdapterError) -> Self {
        match value {
            AdapterError::Tx(_) => 0,
            AdapterError::Rx(_) => 1,
            AdapterError::Timeout => 2,
            AdapterError::Elm327Nok => 3,
            AdapterError::InitError => 4,
            AdapterError::AuthError => 5,
            AdapterError::ParseIntError => 6,
            AdapterError::EcuSpecificError(_) => 7,
        }
    }
}
