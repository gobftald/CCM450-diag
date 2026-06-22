use esp_hal::{
    Async,
    gpio::{AnyPin, Output, Level},
    uart::{AnyUart, Config, Uart, UartRx, UartTx},
};
use embassy_time::Timer;

#[macro_export]
macro_rules! create_gsm {
    ($peripherals:ident) => {
        {
            // Create a configuration layout setting the driver to open-drain
            let pwk_config = OutputConfig::default()
                .with_drive_mode(DriveMode::OpenDrain)
                .with_pull(Pull::None); // A7670C pulls itself up internally
            (
                crate::gsm::GSM::new(
                    $peripherals.UART1.into(),
                    $peripherals.GPIO7.into(),                              // TX
                    $peripherals.GPIO10.into(),                             // RX
                ),
                Output::new($peripherals.GPIO8, Level::High, pwk_config),   // PWK
            )
        }
    }
}

pub struct GSM<'a> {
    pub(crate) rx: UartRx<'a, Async>,
    pub(crate) tx: UartTx<'a, Async>,
}

impl<'a> GSM<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        // configure UART
        let config = Config::default().with_baudrate(115_200);
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

#[embassy_executor::task()]
pub async fn gsm_task(mut gsm: crate::gsm::GSM<'static>, mut pwk_pin: Output<'static>,) {
    let mut buf = [0u8; 32];

    // Trigger the hardware startup pulse sequence
    debug!("Driving A7670C PWK LOW...");
    pwk_pin.set_level(Level::Low);
    Timer::after_millis(1_500).await;
    debug!("Releasing A7670C PWK HIGH...");
    pwk_pin.set_level(Level::High);

    debug!("Waiting for modem firmware initialization...");
    Timer::after_millis(3_000).await;

    gsm.tx.write_async(b"ATI\r\n").await.ok();
    match embassy_time::with_timeout(
        embassy_time::Duration::from_secs(1),
        gsm.rx.read_async(&mut buf)
    ).await {
        Ok(_) => debug!("gps answer {}", buf),
        Err(e) => debug!("gps error {}", e)
    }
}
