use super::*;

use esp_hal::{
    gpio::AnyPin,
    uart::{AnyUart, Config, RxError, TxError, Uart, UartRx, UartTx},
};

pub struct Adapter<'a> {
    pub(crate) rx: UartRx<'a>,
    pub(crate) tx: UartTx<'a>,
}

impl<'a> Adapter<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        // configure UART
        let config = Config::default().with_baudrate(38_400);
        let mut uart = unwrap!(Uart::new(uart, config))
            .with_tx(tx_pin)
            .with_rx(rx_pin);
        uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default().with_cmd_char(b'>'));
        let (rx, tx) = uart.split();

        Self { rx, tx }
    }
}

impl<'a> Adapters for Adapter<'a> {
    async fn connect(&mut self) -> Result<(), TxError> {
        // forwarding only TxError
        self.tx.write_async(b"ATZ\r").await?;
        Ok(())
    }
    async fn write(&mut self, request: &[u8]) -> Result<usize, TxError> {
        self.tx.write_async(request).await
    }

    // there is no conversion between results, so we can accept future directly
    fn read(&mut self, response: &mut [u8]) -> impl Future<Output = Result<usize, RxError>> {
        self.rx.read_async(response, false)
    }
}
