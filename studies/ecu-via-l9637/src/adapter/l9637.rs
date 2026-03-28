use esp_hal::{
    Async,
    gpio::AnyPin,
    uart::{AnyUart, Config, Uart, UartRx, UartTx},
};

use esp_hal::uart::{RxError, TxError};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdapterError {
    TxError(TxError),
    RxError(RxError),
}

impl From<AdapterError> for u8 {
    fn from(value: AdapterError) -> Self {
        match value {
            AdapterError::TxError(_) => 0,
            AdapterError::RxError(_) => 1,
        }
    }
}

use super::Adapters;

pub struct Adapter<'a> {
    pub(crate) rx: UartRx<'a, Async>,
    pub(crate) tx: UartTx<'a, Async>,
}

impl<'a> Adapter<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        Self { rx, tx }
    }
}

impl<'a> Adapters for Adapter<'a> {
    async fn write(&mut self, request: &[u8]) -> Result<usize, AdapterError> {
        self.tx.write_async(request).await.map_err(AdapterError::TxError)
    }

    async fn read(&mut self, response: &mut [u8]) -> Result<usize, AdapterError> {
        self.rx
        //.read_async(response, false)
        .read_async(response)
        .await
        .map_err(AdapterError::RxError)
    }
}

#[macro_export]
macro_rules! create_adapter {
    ($peripherals:ident) => {
        crate::adapter::Adapter::new(
            /*
            $peripherals.UART0.into(),
            $peripherals.GPIO21.into(),
            $peripherals.GPIO20.into(),
                */
            $peripherals.UART1.into(),
            $peripherals.GPIO2.into(),
            $peripherals.GPIO3.into(),
        )
    }
}