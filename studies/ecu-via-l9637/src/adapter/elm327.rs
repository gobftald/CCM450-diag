use esp_hal::{
    Async,
    gpio::AnyPin,
    uart::{AnyUart, Config, Uart, UartRx, UartTx},
};

use embassy_futures::select::{Either, select};

use crate::CHANNEL_ITEM_SIZE;

use esp_hal::uart::{RxError, TxError};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdapterError {
    TxError(TxError),
    RxError(RxError),
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
            AdapterError::TxError(_) => 0,
            AdapterError::RxError(_) => 1,
            AdapterError::Timeout => 2,
            AdapterError::Elm327Nok => 3,
            AdapterError::InitError => 4,
            AdapterError::AuthError => 5,
            AdapterError::ParseIntError => 6,
            AdapterError::EcuSpecificError(_) => 7,
        }
    }
}

pub struct Adapter<'a> {
    pub(crate) rx: UartRx<'a, Async>,
    pub(crate) tx: UartTx<'a, Async>,
}

impl<'a> Adapter<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        // configure UART
        let config = Config::default().with_baudrate(115_200);
        let mut uart = unwrap!(Uart::new(uart, config))
            .into_async()
            .with_tx(tx_pin)
            .with_rx(rx_pin);
        uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default().with_cmd_char(b'>'));
        let (rx, tx) = uart.split();

        Self { rx, tx }
    }

    #[allow(non_snake_case)]
    pub async fn wait_AT_prompt(
        &mut self,
        buf: &mut [u8],
        timeout: u64,
        check_ok: bool,
    ) -> Result<usize, AdapterError> {
        let mut size = 0;
        loop {
            match select(
                //self.rx.read_async(&mut buf[size..], false),
                self.rx.read_async(&mut buf[size..]),
                embassy_time::Timer::after(embassy_time::Duration::from_millis(timeout)),
            )
            .await
            {
                Either::First(result) => {
                    size += result.map_err(AdapterError::RxError)?;
                    trace!("read buf: {:a}", buf[..size]);
                    if buf[size - 1] == b'>' {
                        if !check_ok || buf[size - 5] == b'O' && buf[size - 4] == b'K' {
                            break;
                        } else {
                            error!("Elm327Nok error");
                            return Err(AdapterError::Elm327Nok);
                        }
                    }
                }

                Either::Second(_) => return Err(AdapterError::Timeout),
            }
        }
        //panic!("haho");
        Ok(size)
    }
}

#[allow(unused_must_use)]
impl<'a> Adapter<'a> {
    pub async fn connect(&mut self, secret_key: u32) -> Result<(), AdapterError> {
        let mut buf: [u8; 32] = [0; 32];

        // Warm (Re)Start - ELM327 specific command
        self.tx.write(b"ATWS\r").map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 100, false).await?;

        // echo off - ELM327 specific command
        self.tx.write(b"ATE0\r").map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // set WakeUp Message
        // 0x82   - physical addressing
        //        - message length = 2
        // 0x12   - target address
        // 0xf1   - source address
        // 0x3e   - TesterPresent Request Service ID (first byte of payload)
        // 0x00   - Response is not Required (second byte of payload)
        // checksum will be added by ELM327
        self.tx
            .write(b"AT WM 82 12 F1 3E 00\r")
            .map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // defult is 3 sec (0x92 * 20.48 msec)
        // set WakeUp frequency (4900/20.48 in hex) - ELM327 specific command
        // a bit less then 5sec, since 5sec was  unstable
        self.tx.write(b"AT SW EF\r").map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // set Header
        // 0x81   - physical addressing
        //      - length value should be inserted by ELM327 into this (header format) byte
        // 0x12   - target address
        // 0xf1   - source address
        // this header with the proper length value will be sent by ELM327
        // then the actual payload will be sent
        // finally ELM327 calculate cheksum and wiill send it as the last byte
        self.tx
            .write(b"AT SH 81 12 F1\r")
            .map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 100, true).await?;

        // Fast Init - ELM327 specific command
        self.tx.write(b"AT FI\r").map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 500, true).await?;

        // StartCommunication request
        // 0x81 = startCommunication Request Service Id
        self.tx.write(b"81\r").map_err(AdapterError::TxError)?;
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

        // Authentication
        let mut seed: u32 = 0;
        // this is an additonal experience to the findings I published on cafehusky
        // Secret Key is only valid for Seeds which are even numbers
        // but if you get an even Seed you can authenticate as frequently as you can
        // so the 10 sec mandatory gap time is only valid for failed authentication
        //
        // may be there are Secret Keys for odd Seeds, but I don't want to find them;
        // this authentcation mechanism, looking for even Seed then use it for
        // authentication is enough for me, it causes negligable variance in connection
        // time
        loop {
            // 0x27 - Security Access Request Service ID
            // 0x03 - Access Mode - '03 secure mode Request Seed'
            // only the last bit is important -> 1, 3, 5 ... -> request a seed, even numbers -> error
            self.tx.write(b"27 03\r").map_err(AdapterError::TxError)?;
            self.wait_AT_prompt(&mut buf, 500, false).await?;

            // extract Seed
            #[allow(unused_parens)]
            if (
                // 67 - Security Access Positive Response Service ID
                (buf[0] == b'6' && buf[1] == b'7') &&
                // 03 - Access Mode - '02 secure mode' Request Seed
                (buf[3] == b'0' && buf[4] == b'3')
            ) {
                seed = from_ascii_bytes_to_u16(&buf[6..11])? as u32;
            }
            // in case of Negative response seed is wrong then the authentication will be failed

            if seed.is_multiple_of(2) {
                //
                break;
            }
        }

        // calculate secure key
        let key = (seed * secret_key) % 0x10000;

        buf[0] = b'2'; // Security Access Request Service ID
        buf[4] = b'4'; // 04 - Access Mode - '02 secure mode' Send Key

        from_u16_to_ascii_bytes(key as u16, &mut buf[6..10]);

        buf[10] = b'\r';

        // 0x27 - Security Access Request Service ID
        // 04 - Access Mode - '04 secure mode Send Key'
        // response key calculated based on our Secret Key applied to Seed
        self.tx.write(&buf[..11]).map_err(AdapterError::TxError)?;
        self.wait_AT_prompt(&mut buf, 500, false).await?;

        #[allow(unused_parens)]
        if (
            // 67 - Security Access Positive Response Service ID
            (buf[0] != b'6' || buf[1] != b'7') ||
            // 04 - Access Mode - '02 secure mode' Request Seed Accepted
            (buf[3] != b'0' || buf[4] != b'4')
        ) {
            error!("AuthError");
            return Err(AdapterError::AuthError);
        }

        // Stop Diagnostic Session
        //self.tx.write(b"AT 20\r").map_err(AdapterError::Tx)?;
        //self.wait_AT_prompt(&mut buf, 100, false).await?;

        // increase default timeout - ELM327 specific command
        //self.tx.write(b"AT ST FF\r").map_err(AdapterError::Tx)?;
        //self.wait_AT_prompt(&mut buf, 100, true).await?;
        Ok(())
    }

    pub async fn read_data_by_common_id(
        &mut self,
        ids: &[u8],
        reply: &mut [u8],
    ) -> Result<usize, AdapterError> {
        if ids[0] == 0x25 {
            // ERROR_ECU_SUBFUNCTION_NOT_SUPPORTED__INVALID_FORMAT
            return Err(AdapterError::EcuSpecificError(0x12));
        }

        let mut buf: [u8; CHANNEL_ITEM_SIZE] = [0; CHANNEL_ITEM_SIZE];
        buf[1] = b'2';

        // we have already checked that the input buffer length is even
        for i in 0..ids.len() / 2 {
            buf[0] = b'2';

            from_u16_to_ascii_bytes(
                ids[i * 2] as u16 * 256 + ids[i * 2 + 1] as u16,
                &mut buf[2..6],
            );

            buf[6] = b'\r';

            trace!("read_data_by_common_id is sending: {:a}", &buf[..7]);

            self.tx.write(&buf[..7]).map_err(AdapterError::TxError)?;
            let size = self.wait_AT_prompt(&mut buf, 500, false).await?;

            if buf[0] == b'6' && buf[1] == b'2' {
                let res = from_ascii_bytes_to_u16(&buf[9..14])?;
                reply[2 + i * 2] = (res / 256) as u8;
                reply[2 + i * 2 + 1] = (res % 256) as u8;
            } else {
                return Err(decode_err_status(&mut buf[..size]));
            }
        }

        Ok(ids.len() + 2)
    }

    #[allow(unused_assignments)]
    pub async fn read_diagnostic_trouble_codes_by_status(
        &mut self,
        reply: &mut [u8],
    ) -> Result<usize, AdapterError> {
        // when I detached the ECU the max nmber of read DTC was 12
        // which fits in a 117 bytes length response message
        let mut buf: [u8; 128] = [0; 128];

        // 0x18 - Read Diagnostic Trouble Codes By Status Request Service ID
        // 0x02 - Request 2 byte hex DTC
        // 0xFFFF - All DTCs
        self.tx.write(b"18 02 FF FF\r").map_err(AdapterError::TxError)?;
        let size = self.wait_AT_prompt(&mut buf, 500, false).await?;
        let mut dtc_num = 0;

        if buf[0] == b'5' && buf[1] == b'8' {
            dtc_num = from_ascii_bytes_to_u8(&buf[3..5])? as usize;
            trace!("dtc_num {}", dtc_num);
            // "58 XX " => 6 bytes
            // "XX YY 68 " => 9 bytes (68 is 'h' - means hex)
            // "\r\r>" => 3 bytes
            if 6 + dtc_num * 9 + 3 != size {
                // wrong length of response based on the given number of sent DTCs
                return Err(AdapterError::EcuSpecificError(0));
            }

            for i in 0..dtc_num {
                let dtc = from_ascii_bytes_to_u16(&buf[6 + i * 9..6 + i * 9 + 5])?;
                reply[2 + i * 2] = (dtc / 256) as u8;
                reply[2 + i * 2 + 1] = (dtc % 256) as u8;
            }
            trace!("reply {}", &reply[..dtc_num * 2 + 2]);
        } else {
            return Err(decode_err_status(&mut buf[..size]));
        }

        Ok(dtc_num * 2 + 2)
    }

    pub async fn clear_diagnostic_information(&mut self) -> Result<(), AdapterError> {
        let mut buf: [u8; 16] = [0; 16];

        self.tx.write(b"14 FF FF\r").map_err(AdapterError::TxError)?;
        let size = self.wait_AT_prompt(&mut buf, 500, false).await?;

        if buf[0] != b'5' || buf[1] != b'4' {
            return Err(decode_err_status(&mut buf[..size]));
        }

        Ok(())
    }

    pub async fn raw_request(
        &mut self,
        request: &[u8],
        reply: &mut [u8],
    ) -> Result<usize, AdapterError> {
        let mut buf: [u8; 128] = [0; 128];

        for (i, byte) in request.iter().enumerate() {
            from_u8_to_ascii_bytes(*byte, &mut buf[i * 2..i * 2 + 2])?;
        }
        buf[request.len() * 2] = b'\r';

        self.tx
            .write(&buf[..request.len() * 2 + 1])
            .map_err(AdapterError::TxError)?;
        let size = self.wait_AT_prompt(&mut buf, 500, false).await?;

        let mut i = 0;
        let mut flag = false;
        for j in 0..size - 4 {
            if buf[j] == b' ' || flag {
                flag = false;
                continue;
            }
            reply[2 + i] = from_ascii_bytes_to_u8(&buf[j..j + 2])?;
            flag = true;
            i += 1;
        }

        Ok(i + 2)
    }

    pub async fn write(&mut self, request: &[u8]) -> Result<usize, AdapterError> {
        self.tx.write_async(request).await.map_err(AdapterError::TxError)
    }

    pub async fn read(&mut self, response: &mut [u8]) -> Result<usize, AdapterError> {
        self.rx
        //.read_async(response, false)
        .read_async(response)
        .await
        .map_err(AdapterError::RxError)
    }
}

#[allow(clippy::map_identity)]
#[allow(clippy::needless_return)]
fn decode_err_status(reply: &mut [u8]) -> AdapterError {
    if compare_bytes(b"NO DATA\r\r>", reply) {
        return AdapterError::Timeout;
    }
    if reply[0] == b'7' && reply[1] == b'F' {
        let res = unwrap!(from_ascii_bytes_to_u8(&reply[6..8]).map_err(|err| return err));
        AdapterError::EcuSpecificError(res)
    } else {
        // O means OK in Ecu's status values, but here it is only
        // used to indicate errors, to indicate an unknown error
        // when we cannot interpret the error response
        AdapterError::EcuSpecificError(0)
    }
}

fn char_to_num(char: u8) -> Result<u8, AdapterError> {
    if char.is_ascii_digit() {
        Ok(char - b'0')
    } else if (b'A'..=b'F').contains(&char) {
        Ok(char - b'A' + 10)
    } else {
        Err(AdapterError::ParseIntError)
    }
}

pub fn from_ascii_bytes_to_u8(buf: &[u8]) -> Result<u8, AdapterError> {
    if buf.len() != 2 {
        return Err(AdapterError::ParseIntError);
    }

    let mut val = char_to_num(buf[0])? << 4;
    val += char_to_num(buf[1])?;

    Ok(val)
}

pub fn from_ascii_bytes_to_u16(buf: &[u8]) -> Result<u16, AdapterError> {
    if buf.len() < 4 {
        return Err(AdapterError::ParseIntError);
    }

    let mut i = 0;
    let mut val = (from_ascii_bytes_to_u8(&buf[i..i + 2])? as u16) << 8;
    i += 2;
    while buf[i] == b' ' {
        i += 1;
        continue;
    }
    val += from_ascii_bytes_to_u8(&buf[i..i + 2])? as u16;

    Ok(val)
}

pub fn from_u8_to_ascii_bytes(value: u8, buf: &mut [u8]) -> Result<(), AdapterError> {
    if buf.len() < 2 {
        return Err(AdapterError::ParseIntError);
    }

    let digits = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F',
    ];
    buf[0] = digits[(value / 16) as usize];
    buf[1] = digits[(value % 16) as usize];

    Ok(())
}

pub fn from_u16_to_ascii_bytes(value: u16, buf: &mut [u8]) -> Result<(), AdapterError> {
    if buf.len() < 4 {
        return Err(AdapterError::ParseIntError);
    }

    from_u8_to_ascii_bytes((value / 256) as u8, &mut buf[0..2])?;
    from_u8_to_ascii_bytes((value % 256) as u8, &mut buf[2..4])?;

    Ok(())
}

pub fn compare_bytes(one: &[u8], another: &[u8]) -> bool {
    if one.len() != another.len() {
        return false;
    }
    for (i, one) in one.iter().enumerate() {
        if *one != another[i] {
            return false;
        }
    }
    true
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
