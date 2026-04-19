use esp_hal::{
    Async, gpio::AnyPin, uart::{AnyUart, Config, Uart, UartRx, UartTx}
};

use esp_hal::uart::{RxError, TxError};

const KWP2000_BAUDRATE: u32 = 10_400;

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
        let config = Config::default()
            .with_baudrate(KWP2000_BAUDRATE);

        let (rx, tx) = unwrap!(Uart::new(any_uart, config))
            .into_async()
            .with_tx(tx_pin)
            .with_rx(rx_pin)
            .split();

        Self { rx, tx }
    }
}

impl<'a> Adapters for Adapter<'a> {
    async fn transmit(&mut self, request: &mut [u8]) -> Result<(), AdapterError> {
        trace!("transmit {:x}", request);
        self.tx.write_async(request).await.map_err(AdapterError::TxError)?;

        // K-Line echo elimination
        self.rx.read_exact_async(request).await.map_err(AdapterError::RxError)
    }

    async fn receive(&mut self, response: &mut [u8]) -> Result<usize, AdapterError> {
        self.rx.read_exact_async(&mut response[..1]).await.map_err(AdapterError::RxError)?;

        // calculate remaining size of message from the leading bytes
        // OK I know this is too specific for adapter level, even for KWP2000 protocol
        // but for now this is the most comfortable
        let len = ((response[0] - 0x80) + 4) as usize;
        self.rx.read_exact_async(&mut response[1..len]).await.map_err(AdapterError::RxError)?;
        trace!("receive {:x}", &response[..len]);
        Ok(len)
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

            // let the UART pins far from wifi antenna and oscillator
            // in a position wrap/breadboard wires don't cross antenna
            $peripherals.UART0.into(),
            $peripherals.GPIO4.into(),
            $peripherals.GPIO3.into(),
        )
    }
}