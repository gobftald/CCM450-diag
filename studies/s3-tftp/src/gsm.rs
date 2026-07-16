use core::ptr::copy_nonoverlapping as cpn;

use esp_hal::{
    Async,
    gpio::{AnyPin, Level, Output},
    uart::{AnyUart, Config, RxError, RxConfig, Uart, UartRx, UartTx},
};
use embassy_time::{Timer, Duration};
use embassy_futures::select::{select, Either};

use crate::gps::{GPS_DATA, GPS_UPDATED};

pub static mut GSM_OK: [u8; 1] = [b' '];

#[macro_export]
macro_rules! create_gsm_uart {
    ($peripherals:ident) => {
        {
            // Create a configuration layout setting the driver to open-drain
            let pwk_config = OutputConfig::default()
                // OpenDrain means the active HIGH switch inside the ESP32-S3 stays disabled
                // When set LOW: The ESP32-S3 actively forces the line to 0V (Ground).
                // When set HIGH: The active switch turns off, and the weak internal Pull::Up resistor
                // gently lifts the line to 3.3V.
                .with_drive_mode(DriveMode::PushPull)
                .with_pull(Pull::None);
            (
                crate::gsm::GsmUart::new(
                    $peripherals.UART1.into(),
                    $peripherals.GPIO7.into(),                              // TX
                    $peripherals.GPIO10.into(),                             // RX
                ),
                Output::new($peripherals.GPIO8, Level::High, pwk_config),   // PWK
            )
        }
    }
}

pub struct GsmUart<'a> {
    rx: UartRx<'a, Async>,
    tx: UartTx<'a, Async>,
}

impl<'a> GsmUart<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        // configure UART
        let config = Config::default()
            //.with_baudrate(115_200)
            .with_baudrate(9_600)
            .with_rx(
                // explicitely disable timeout
                RxConfig::default().with_timeout_none()
            );
        let mut uart = unwrap!(Uart::new(uart, config))
            .into_async()
            .with_tx(tx_pin)
            .with_rx(rx_pin);
        uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default()
            .with_pre_idle_count(0)
            .with_post_idle_count(0)
            .with_cmd_char(b'\n'));
        let (rx, tx) = uart.split();

        Self { rx, tx }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(PartialEq)]
pub enum ModemState {
    PowerOnReset,       // switch off/on with pwk pin
    RadioActivating,    // Passively awaiting network initialization
    Attached,           // Connected to tower, ready to open IP stack
    SocketConfiguring,  // Opening UDP channel
    SocketReady,        // Idle state, ready to transmit data payloads
                        // 5 sec periodic timeout for waiting refreshed GPS data
    DataTransmitting,   // Payload sent, awaiting confirmation
}

async fn read<'a>(gsm: &mut GsmUart<'_>, mut buf: &'a mut [u8]) -> Result<&'a [u8], RxError> {
    gsm.rx.wait_for_buffered_data(5, buf.len(), false).await?;
    let size = gsm.rx.read(&mut buf)?;
    trace!("*** gsm {} {:a}", size, buf[2..size - 2 ]);

    // the format of unsolicited messages
    Ok(&buf[2..size - 2 ])
}

async fn switch_modem_off(pwk_pin: &mut Output<'static>) {
    trace!("pwk_pin.set_level(Level::Low)");
    pwk_pin.set_level(Level::Low);
    Timer::after_millis(2_500).await;
    trace!("pwk_pin.set_level(Level::High)");
    pwk_pin.set_level(Level::High);
}

async fn switch_modem_on(pwk_pin: &mut Output<'static>) {
    trace!("pwk_pin.set_level(Level::Low)");
    pwk_pin.set_level(Level::Low);
    Timer::after_millis(200).await;
    trace!("pwk_pin.set_level(Level::High)");
    pwk_pin.set_level(Level::High);
}

#[embassy_executor::task()]
pub async fn gsm_task(mut gsm: crate::gsm::GsmUart<'static>, mut pwk_pin: Output<'static>) {
    let mut buf: [u8; 128] = [0; 128];
    let mut udp_send_literal = *b"AT+CIPSEND=0,,\"000.000.000.000\",19924\r\n";
    let udp_send = &mut udp_send_literal;
    let mut udp_send_len: usize = 0;
    let mut state = ModemState::PowerOnReset;
    let mut once_active = false;
    let mut timeout = Duration::from_secs(60);
    let mut xfer_ok = false;
    let mut bad_xfer: u32 = 0;

    // get GPS_UPDATES watch receiver
    let mut gps_updated = unwrap!(GPS_UPDATED.receiver());

    /*
    // reset
    let _ = gsm.tx.write_async(b"AT+CRESET\r\n").await;
    let _ = gsm.tx.write_async(b"ATE0\r\n").await;
    */

    loop {
        // State machine
        match state {
            ModemState::PowerOnReset => {
                switch_modem_off(&mut pwk_pin).await;
                Timer::after_millis(1_000).await;
                switch_modem_on(&mut pwk_pin).await;

                once_active = false;
                state = ModemState::RadioActivating;
                xfer_ok = false;
                bad_xfer = 0;
                unsafe { GSM_OK[0] = b' '; }
            }
            ModemState::RadioActivating => {
                // Passive wait state. Wating for b"+CGEV: NW PDN ACT1"
            }
            ModemState::Attached => {
                if once_active {
                    debug!("*** Network stable. Sending all initialization commands in sequence...");

                    // Echo off
                    let _ = gsm.tx.write_async(b"ATE0\r\n").await;
                    Timer::after_millis(200).await;

                    // Configure the APN
                    let _ = gsm.tx.write_async(b"AT+CGDCONT=1,\"IP\",\"bicsapn\"\r\n").await;
                    Timer::after_millis(200).await;

                    // Attach to the GPRS Data Network.
                    let _ = gsm.tx.write_async(b"AT+CGATT=1\r\n").await;
                    Timer::after_millis(200).await;

                    // Activate the PDP Context.
                    let _ = gsm.tx.write_async(b"AT+CGACT=1,1\r\n").await;
                    Timer::after_millis(200).await;

                    // Open the TCP/IP Network Stack
                    let _ = gsm.tx.write_async(b"AT+NETOPEN\r\n").await;

                    // Mandatory 2-second stabilization delay to let internal virtual routing maps build
                    Timer::after_secs(2).await;

                    // Get IP address.
                    //let _ = gsm.tx.write_async(b"AT+CDNSGIP=\"ivancsics.hu\"\r\n").await;
                    let _ = gsm.tx.write_async(b"AT+CDNSGIP=\"gobftald.ddns.net\"\r\n").await;

                    // Open UDP socket.
                    let _ = gsm.tx.write_async(b"AT+CIPOPEN=0,\"UDP\",,,1234\r\n").await;
                    Timer::after_millis(200).await;

                    state = ModemState::SocketConfiguring;
                }
            }
            ModemState::SocketConfiguring => {
                // Passive wait state. No outbound writes happen here.
                // Waiting for success (b"+CIPOPEN: 0,0") or 10 secs timout
            }
            ModemState::SocketReady => {
                // start timer
            }
            ModemState::DataTransmitting => {
                xfer_ok = false;

                unsafe {
                    let record = core::slice::from_raw_parts(
                        &raw const GPS_DATA as *const u8,
                        49,
                    );

                    //let _ = gsm.tx.write_async(b"AT+CIPSEND=0,,\"46.139.107.93\",1234\r\n").await;
                    let _ = gsm.tx.write_async(&udp_send[..udp_send_len]).await;
                    Timer::after_millis(100).await;
                    let _ = gsm.tx.write_async(unwrap!(record.try_into())).await;
                    let _ = gsm.tx.write_async(b"\r\n\x1A").await;
                    debug!("*** gsm sending {:a}", record);
                }

                state = ModemState::SocketReady;
            }
        }

        // Dynamic Timeout Selection
        match state {
            // If we caught an ACT message, we want to wait for one 'event free' 2 seconds more.
            // This needs to handle DEACT -> ... -> ACTACT transient which is typical at power on
            // in case of a roaming SIM card (before a finally Confirmed Activation)
            ModemState::Attached if !once_active => {
                trace!("*** gsm set 2x2 secs timeout");
                timeout = Duration::from_secs(2)
            }

            // If we don't have an ACT yet, we wait up to 30 seconds.
            //
            // but we don't want to manage the DEACT -> ... -> ACT
            // situation after a successful activation anymore
            ModemState::RadioActivating if once_active => {
                trace!("*** gsm set 30 secs timeout");
                timeout = Duration::from_secs(30)
            }

            // timeout if nothing happened during SocketConfiguring
            // commands, neither any state change nor success
            ModemState::SocketConfiguring => {
                trace!("*** gsm set 10 secs timeout");
                timeout = Duration::from_secs(10)
            }

            ModemState::SocketReady => {
                trace!("*** gsm start waiting for gps data");
                // transmission takes about 230ms in 9600 buad
                // so we deduct this time from the 5s cycyles
                timeout = Duration::from_millis(4_770);
            }

            // Otherwise we set 60 second watchdog timout. But don't
            // override transients' timeouts before cofirmed activation
            _ if once_active => {
                trace!("*** gsm set 60 secs timeout");
                timeout = Duration::from_secs(60)
            }
            _ => {}
        }

        match select(read(&mut gsm, &mut buf), Timer::after(timeout)).await {
            Either::First(result) => {
                if let Ok(msg) = result {
                    // Get DNS for ivancsics.hu
                    if msg.len() > 7 &&
                       &msg[..8] == b"+CDNSGIP" &&
                       msg.len() > 33 &&
                       &msg[..33] == b"+CDNSGIP: 1,\"gobftald.ddns.net\",\"" {
                        debug!("*** gsm DNS resolution {:a}", msg);
                        unsafe {
                            cpn(
                                &msg[33] as *const u8,
                                &raw mut udp_send[15],
                                msg.len() - 34
                            );
                        }
                        udp_send_len = 15 + msg.len() - 34;
                        udp_send[udp_send_len] = b'\"';
                        udp_send[udp_send_len + 1] = b',';
                        let port = env!("GSM_PORT").as_bytes();
                        unsafe {
                            cpn(
                                port.as_ptr(),
                                &raw mut udp_send[udp_send_len + 2],
                                port.len()
                            );
                        }

                        udp_send_len += port.len() + 2;
                        udp_send[udp_send_len] = b'\r';
                        udp_send[udp_send_len + 1] = b'\n';
                        udp_send_len += 2;

                        debug!("*** gsm udp_send {:a}", &udp_send[..udp_send_len]);
                    }
                    match msg {
                        b"+CGEV: ME DETACH" | b"+CGEV: NW DETACH" => {
                            if once_active {
                                debug!("*** gsm Critical Network Drop. Rewinding to PowerOnReset...");
                                state = ModemState::PowerOnReset;
                            };
                            // else ignore it, since it can happen during power on
                        }
                        b"+CGEV: NW PDN DEACT 1" => {
                            debug!("*** gsm Data Connection Drop. Re-attaching...");
                            state = ModemState::RadioActivating;
                        }
                        b"+CGEV: NW PDN ACT 1" => {
                            if state == ModemState::RadioActivating {
                                debug!("*** gsm Network Registration.");
                                state = ModemState::Attached;
                            }
                        }
                        b"+CIPOPEN: 0,0" => {
                            if state == ModemState::SocketConfiguring {
                                debug!("*** gsm UDP Socket verification confirmed!");
                                state = ModemState::SocketReady;
                            }
                        }
                        // here is not leading \r\n so b"+C" was stripped
                        b"IPSEND: 0,51,51" => {
                            debug!("*** gsm EOF Data Transmission");
                            xfer_ok = true;
                            unsafe { GSM_OK[0] = b'G'; }
                        }
                        _ => {
                            trace!("*** gsm unhandled msg: {:a}", msg);
                        }
                    }
                }
            }
            Either::Second(_) => {
                match state {
                    ModemState::RadioActivating => {
                        // If the modem is stuck in this state for more than 30 seconds
                        // without triggering a clean registration,  break the freeze
                        // and force a hardware reset.
                        debug!("*** gsm Modem cannot register. Rewinding to PowerOnReset...");
                        state = ModemState::PowerOnReset;
                    }
                    ModemState::Attached => {
                        debug!("*** gsm Confirm Network Registration");
                        once_active = true;
                    }
                    ModemState::SocketConfiguring => {
                        // If the modem is stuck in this state for more than 10 seconds
                        // without successfuly open an UDP socket, we handle it simply
                        // and brutally, forcing a hardware reset
                        debug!("*** gsm Modem cannot open UDP. Rewinding to PowerOnReset...");
                        state = ModemState::PowerOnReset;
                    }
                    ModemState::SocketReady => {
                        if !xfer_ok {
                            bad_xfer += 1;
                            unsafe { GSM_OK[0] = b' '; }
                        } else {
                            xfer_ok = false;
                            bad_xfer = 0;
                        }

                        if bad_xfer < 10 {
                            let _ = gps_updated.try_changed().map(|_| state = ModemState::DataTransmitting);
                        } else {
                            debug!("gsm Too many unsuccesfull xfer. Rewinding to PowerOnReset...");
                            state = ModemState::PowerOnReset;
                        }
                    }

                    _ => {
                        // theoretically it cannot be happened
                        debug!("*** gsm Watchdog timout happened - state is {}", state);
                    }
                }
            }
        }
    }
}
