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
    async fn write_async(&mut self, request: &[u8]) -> Result<usize, TxError> {
        self.tx.write_async(request).await
    }

    async fn read_async(&mut self, response: &mut [u8]) -> Result<usize, RxError> {
        trace!("ecu_adapter/elm327: self.rx.read_async(response, false).await");
        let res = self.rx.read_async(response, false).await;
        trace!("ecu_adapter/elm327: self.rx.read_async(response, false).await awaken");
        res
    }
}
