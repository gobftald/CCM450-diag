use esp_hal::{
    gpio::AnyPin,
    uart::{AnyUart, Config, Uart, UartRx, UartTx},
};

use embassy_futures::select::{Either, select};

use super::{AdapterError, Adapters};

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

    async fn wait_for_prompt(&mut self, check_ok: bool, timeout: u64) -> Result<(), AdapterError> {
        let mut buf: [u8; 32] = [0; 32];
        loop {
            match select(
                self.rx.read_async(&mut buf, false),
                embassy_time::Timer::after(embassy_time::Duration::from_millis(timeout)),
            )
            .await
            {
                Either::First(result) => {
                    trace!("wait for prompt buf: {}", buf);
                    let size = result.map_err(|err| {
                        error!("read_async error {}", err);
                        AdapterError::Rx(err)
                    })?;

                    if buf[size - 1] == b'>' {
                        if !check_ok {
                            break;
                        } else if buf[size - 5] == b'O' && buf[size - 4] == b'K' {
                            break;
                        }
                    }
                }

                Either::Second(_) => {
                    error!("#### Uart timeout error");
                    return Err(AdapterError::Timeout);
                }
            }
        }
        Ok(())
    }
}

#[allow(unused_must_use)]
impl<'a> Adapters for Adapter<'a> {
    async fn connect(&mut self) -> Result<(), AdapterError> {
        // reset
        self.tx.write(b"ATZ\r").map_err(AdapterError::Tx);
        self.wait_for_prompt(false, 1000).await?;

        // echo off
        self.tx.write(b"ATE0\r").map_err(AdapterError::Tx);
        self.wait_for_prompt(true, 500).await?;

        // set WakeUp Message
        self.tx
            .write(b"AT WM 82 12 F1 3E 01\r")
            .map_err(AdapterError::Tx);
        self.wait_for_prompt(true, 500).await?;

        // set WakeUp frequency (5000/20 in hex)
        self.tx.write(b"AT SW FA\r").map_err(AdapterError::Tx);
        self.wait_for_prompt(true, 500).await?;

        // set Header
        self.tx.write(b"AT SH 81 12 F1\r").map_err(AdapterError::Tx);
        self.wait_for_prompt(true, 500).await?;

        // Fast Init
        self.tx.write(b"AT FI\r").map_err(AdapterError::Tx);
        self.wait_for_prompt(false, 1000).await?;

        Ok(())
    }
    async fn write(&mut self, request: &[u8]) -> Result<usize, AdapterError> {
        self.tx.write_async(request).await.map_err(AdapterError::Tx)
    }

    async fn read(&mut self, response: &mut [u8]) -> Result<usize, AdapterError> {
        self.rx
            .read_async(response, false)
            .await
            .map_err(AdapterError::Rx)
    }
}
