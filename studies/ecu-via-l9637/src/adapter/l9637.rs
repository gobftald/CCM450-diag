use esp_hal::{
    Async, gpio::AnyPin, peripherals, uart::{AnyUart, Config, Uart, UartRx, UartTx}
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
    pub fn new(any_uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        // configure UART
        let config = Config::default().with_baudrate(10_400);
        let mut uart;
        unsafe {
            uart = unwrap!(Uart::new(any_uart, config))
                .into_async()
                .with_tx(tx_pin.clone_unchecked())
                .with_rx(rx_pin);
        }
        uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default().with_cmd_char(b'>'));
        let (rx, tx) = uart.split();

        Self { rx, tx }
    }
}

impl<'a> Adapters for Adapter<'a> {
    async fn transmit(&mut self, request: &[u8]) -> Result<usize, AdapterError> {
        self.tx.write_async(request).await.map_err(AdapterError::TxError)
    }

    async fn receive(&mut self, response: &mut [u8]) -> Result<usize, AdapterError> {
        self.rx
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