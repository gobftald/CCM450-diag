#![allow(non_snake_case)]

use core::char::from_u32_unchecked;

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

    async fn wait_AT_prompt(
        &mut self,
        buf: &mut [u8],
        timeout: u64,
        check_ok: bool,
    ) -> Result<usize, AdapterError> {
        let mut size = 0;
        loop {
            match select(
                self.rx.read_async(&mut buf[size..], false),
                embassy_time::Timer::after(embassy_time::Duration::from_millis(timeout)),
            )
            .await
            {
                Either::First(result) => {
                    size += result.map_err(|err| {
                        error!("elm327 read_async error {}", err);
                        AdapterError::Rx(err)
                    })?;

                    trace!("wait for prompt buf: {:a}", buf[..size]);

                    if buf[size - 1] == b'>' {
                        if !check_ok || buf[size - 5] == b'O' && buf[size - 4] == b'K' {
                            break;
                        } else {
                            error!("Elm327Nok error");
                            return Err(AdapterError::Elm327Nok);
                        }
                    }
                }

                Either::Second(_) => {
                    error!("#### Uart timeout error");
                    return Err(AdapterError::Timeout);
                }
            }
        }
        Ok(size)
    }
}

#[allow(unused_must_use)]
impl<'a> Adapters for Adapter<'a> {
    async fn connect(&mut self, secret_key: u32) -> Result<(), AdapterError> {
        let mut buf: [u8; 32] = [0; 32];

        // reset - ELM327 specific command
        self.tx.write(b"ATZ\r").map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 1000, false).await?;

        // echo off - ELM327 specific command
        self.tx.write(b"ATE0\r").map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // set WakeUp Message
        // 0x82   - physical addressing
        //      - message length = 2
        // 0x12   - target address
        // 0xf2   - source address
        // 0x3e   - TesterPresent Request Service ID (first byte of payload)
        // 0x01   - Response Required (default) (second byte of payload)
        // checksum will be added by ELM327
        self.tx
            .write(b"AT WM 82 12 F1 3E 01\r")
            .map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // set WakeUp frequency (5000/20 in hex) - ELM327 specific command
        self.tx.write(b"AT SW FA\r").map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // set Header
        // 0x81   - physical addressing
        //      - length value should be inserted by ELM327 into this (header format) byte
        // 0x12   - target address
        // 0xf2   - source address
        // this header with the proper length value will be sent by ELM327
        // then the actual payload will be sent
        // finally ELM327 calculate cheksum and wiill send it as the last byte
        self.tx.write(b"AT SH 81 12 F1\r").map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // Fast Init - ELM327 specific command
        self.tx.write(b"AT FI\r").map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 500, true).await?;

        // StartCommunication request
        // 0x81 = startCommunication Request Service Id
        self.tx.write(b"81\r").map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 500, false).await?;

        #[allow(unused_parens)]
        // StartCommunication response
        if (
            // C1 - startCommunication Positive Response Service Id
            (buf[0] != b'C' || buf[1] != b'1') ||
            // 5B - Key byte 1 (Low byte) - 0101 1011
            // bit7 = Parity = 0
            // bit6 = 1 <- fix 1
            // bit5 = TP1 = 0 -> extended timing parameter set
            // bit4 = TP0 = 1 -> extended timing parameter set
            // bit3 = HB1 = 1 -> Target/Source address in header supported
            // bit2 = HB0 = 0 -> 1 byte header not supported
            // bit1 = AL1 = 1 -> additional length byte supported
            // bit0 = AL0 = 1 -> length information in format byte supported
            //
            // we set up default header (sent always by ELM327 wraping the given payload)
            // accordingly in "AT SH" command above, so periodic WakeUp messages
            // can start after startCommunication Request immediately 
            (buf[3] != b'5' || buf[4] != b'B') ||
            // 8F - key byte 2 is a fix 0x8F
            (buf[6] != b'8' || buf[7] != b'F')
        ) {
            error!("InitError");
            return Err(AdapterError::InitError);
        }

        let mut seed: u32 = 0;
        // this is an additonal experience to the findings I published on cafehusky
        // Secret Key is only valid for Seeds which are even numbers
        // but if you get an even Seed you can authenticate as frequently as you can
        // so the 10 sec mandatory gap time is only valid for failed authentication
        //
        // may be there are Secret Keys for odd Seeds, but I don't want to find them
        // this authentcation mechanism, looking for even Seed then use it for
        // authentication is enough for me, it causes negligable variance in connection
        // time
        loop {
            // 0x27 - Security Access Request Service ID
            // 0x03 - Access Mode - '02 secure mode' Request Seed
            self.tx.write(b"27 03\r").map_err(AdapterError::Tx);
            self.wait_AT_prompt(&mut buf, 500, false).await?;

            // extract Seed
            #[allow(unused_parens)]
            if (
                // 67 - Security Access Positive Response Service ID
                (buf[0] == b'6' || buf[1] == b'7') ||
                // 03 - Access Mode - '02 secure mode' Request Seed
                (buf[3] == b'0' || buf[4] == b'3')
            ) {
                seed += ((buf[6] as char).to_digit(16).unwrap() * 4096);
                seed += ((buf[7] as char).to_digit(16).unwrap() * 256);
                seed += ((buf[9] as char).to_digit(16).unwrap() * 16);
                seed += (buf[10] as char).to_digit(16).unwrap();
            }
            // in case of Negative response seed is wrong then the authentication will be failed

            if seed.is_multiple_of(2) {
                break;
            } else {
                seed = 0;
            }
        }

        // calculate secure key
        let mut key = (seed * secret_key) % 0x10000;

        buf[0] = b'2'; // Security Access Request Service ID
        buf[4] = b'4'; // 04 - Access Mode - '04 secure mode' Send Key

        use core::char::from_digit;
        let mut digit = key / 4096;
        key %= 4096;
        buf[6] = from_digit(digit, 16).unwrap().to_ascii_uppercase() as u8;
        digit = key / 256;
        key %= 256;
        buf[7] = from_digit(digit, 16).unwrap().to_ascii_uppercase() as u8;
        digit = key / 16;
        key %= 16;
        buf[9] = from_digit(digit, 16).unwrap().to_ascii_uppercase() as u8;
        buf[10] = from_digit(key, 16).unwrap().to_ascii_uppercase() as u8;

        // 0x27 - Security Access Request Service ID
        // 04 - Access Mode - '04 secure mode' Send Key
        // response key calculated based on our Secret Key applied to Seed
        self.tx.write(&buf[..13]).map_err(AdapterError::Tx);
        self.wait_AT_prompt(&mut buf, 500, false).await?;

        #[allow(unused_parens)]
        if (
            // 67 - Security Access Positive Response Service ID
            (buf[0] != b'6' || buf[1] != b'7') ||
            // 04 - Access Mode - '02 secure mode' Request Seed
            (buf[3] != b'0' || buf[4] != b'4')
        ) {
            error!("AuthError");
            return Err(AdapterError::AuthError);
        }

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
