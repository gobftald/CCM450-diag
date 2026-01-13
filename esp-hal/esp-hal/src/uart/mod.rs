//! # Universal Asynchronous Receiver/Transmitter (UART)
//!
//! ## Overview
//!
//! The UART is a hardware peripheral which handles communication using serial
//! communication interfaces, such as RS232 and RS485. This peripheral provides!
//! a cheap and ubiquitous method for full- and half-duplex communication
//! between devices.
//!
//! Depending on your device, two or more UART controllers are available for
//! use, all of which can be configured and used in the same way. All UART
//! controllers are compatible with UART-enabled devices from various
//! manufacturers, and can also support Infrared Data Association (IrDA)
//! protocols.
//!
//! ## Configuration
//!
//! Each UART controller is individually configurable, and the usual setting
//! such as baud rate, data bits, parity, and stop bits can easily be
//! configured. Additionally, the receive (RX) and transmit (TX) pins need to
//! be specified.
//!
//! The UART controller can be configured to invert the polarity of the pins.
//! This is achieved by inverting the desired pins, and then constructing the
//! UART instance using the inverted pins.

// 44
use core::task::Poll;

// 48
use enumset::{EnumSet, EnumSetType};

// 51
use crate::{
    asynch::AtomicWaker,
    clock::Clocks,
    gpio::{
        InputConfig, InputSignal, OutputConfig, OutputSignal, PinGuard, Pull,
        interconnect::{PeripheralInput, PeripheralOutput},
    },
    interrupt::InterruptHandler,
    pac::uart0::RegisterBlock,
    peripherals::Interrupt,
    private::OnDrop,
    system::{Cpu, /*PeripheralClockControl,*/ PeripheralGuard},
};

/// UART RX Error
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 70
pub enum RxError {
    /// An RX FIFO overflow happened.
    ///
    /// This error occurs when RX FIFO is full and a new byte is received. The
    /// RX FIFO is then automatically reset by the driver.
    FifoOverflowed,

    /// A glitch was detected on the RX line.
    ///
    /// This error occurs when an unexpected or erroneous signal (glitch) is
    /// detected on the UART RX line, which could lead to incorrect data
    /// reception.
    GlitchOccurred,

    /// A framing error was detected on the RX line.
    ///
    /// This error occurs when the received data does not conform to the
    /// expected UART frame format.
    FrameFormatViolated,

    /// A parity error was detected on the RX line.
    ///
    /// This error occurs when the parity bit in the received data does not
    /// match the expected parity configuration.
    ParityMismatch,
}

/// UART TX Error
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 123
pub enum TxError {}

/// UART clock source
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 143
pub enum ClockSource {
    /// APB_CLK clock source
    #[default]
    Apb,
    /// RC_FAST_CLK clock source (17.5 MHz)
    RcFast,
}

/// Number of data bits
///
/// This enum represents the various configurations for the number of data
/// bits used in UART communication. The number of data bits defines the
/// length of each transmitted or received data frame.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 166
pub enum DataBits {
    /// 5 data bits per frame.
    _5,
    /// 6 data bits per frame.
    _6,
    /// 7 data bits per frame.
    _7,
    /// 8 data bits per frame.
    #[default]
    _8,
}

/// Parity check
///
/// Parity is a form of error detection in UART communication, used to
/// ensure that the data has not been corrupted during transmission. The
/// parity bit is added to the data bits to make the number of 1-bits
/// either even or odd.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 186
pub enum Parity {
    /// No parity bit is used.
    #[default]
    None,
    /// Even parity: the parity bit is set to make the total number of
    /// 1-bits even.
    Even,
    /// Odd parity: the parity bit is set to make the total number of 1-bits
    /// odd.
    Odd,
}

/// Number of stop bits
///
/// The stop bit(s) signal the end of a data packet in UART communication.
/// This enum defines the possible configurations for the number of stop
/// bits.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 205
pub enum StopBits {
    /// 1 stop bit.
    #[default]
    _1,
    /// 1.5 stop bits.
    _1p5,
    /// 2 stop bits.
    _2,
}

/// UART Configuration
//#[derive(Debug, Clone, Copy, procmacros::BuilderLite)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 292
pub struct Config {
    /// The baud rate (speed) of the UART communication in bits per second
    /// (bps).
    baudrate: u32,
    /// Number of data bits in each frame (5, 6, 7, or 8 bits).
    data_bits: DataBits,
    /// Parity setting (None, Even, or Odd).
    parity: Parity,
    /// Number of stop bits in each frame (1, 1.5, or 2 bits).
    stop_bits: StopBits,
    /// Clock source used by the UART peripheral.
    clock_source: ClockSource,
    /// UART Receive part configuration.
    rx: RxConfig,
    /// UART Transmit part configuration.
    tx: TxConfig,
}

// 321
impl Default for Config {
    fn default() -> Config {
        Config {
            rx: RxConfig::default(),
            tx: TxConfig::default(),
            baudrate: 115_200,
            //baudrate_tolerance: BaudrateTolerance::default(),
            data_bits: Default::default(),
            parity: Default::default(),
            stop_bits: Default::default(),
            //sw_flow_ctrl: Default::default(),
            //hw_flow_ctrl: Default::default(),
            clock_source: Default::default(),
        }
    }
}

// 338
impl Config {
    // 339
    fn validate(&self) -> Result<(), ConfigError> {
        /*
        if let BaudrateTolerance::ErrorPercent(percentage) = self.baudrate_tolerance {
            assert!(percentage > 0 && percentage <= 100);
        }
        */

        // Max supported baud rate is 5Mbaud
        if self.baudrate == 0 || self.baudrate > 5_000_000 {
            return Err(ConfigError::UnsupportedBaudrate);
        }
        Ok(())
    }

    pub fn with_baudrate(mut self, baudrate: u32) -> Self {
        self.baudrate = baudrate;
        self
    }
}

/// UART Receive part configuration.
//#[derive(Debug, Clone, Copy, procmacros::BuilderLite)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 356
pub struct RxConfig {
    /// Threshold level at which the RX FIFO is considered full.
    fifo_full_threshold: u16,
    /// Optional timeout value for RX operations.
    timeout: Option<u8>,
}

// 363
impl Default for RxConfig {
    fn default() -> RxConfig {
        RxConfig {
            // see <https://github.com/espressif/esp-idf/blob/8760e6d2a/components/esp_driver_uart/src/uart.c#L61>
            fifo_full_threshold: 120,
            // see <https://github.com/espressif/esp-idf/blob/8760e6d2a/components/esp_driver_uart/src/uart.c#L63>
            timeout: Some(10),
        }
    }
}

/// UART Transmit part configuration.
//#[derive(Debug, Clone, Copy, procmacros::BuilderLite)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 378
pub struct TxConfig {
    /// Threshold level at which the TX FIFO is considered empty.
    fifo_empty_threshold: u16,
}

// 383
impl Default for TxConfig {
    fn default() -> TxConfig {
        TxConfig {
            // see <https://github.com/espressif/esp-idf/blob/8760e6d2a/components/esp_driver_uart/src/uart.c#L59>
            fifo_empty_threshold: 10,
        }
    }
}

/// Configuration for the AT-CMD detection functionality
//#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, procmacros::BuilderLite)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 397
pub struct AtCmdConfig {
    /// Optional idle time before the AT command detection begins, in clock
    /// cycles.
    pre_idle_count: Option<u16>,
    /// Optional idle time after the AT command detection ends, in clock
    /// cycles.
    post_idle_count: Option<u16>,
    /// Optional timeout between bytes in the AT command, in clock
    /// cycles.
    gap_timeout: Option<u16>,
    /// The byte (character) that triggers the AT command detection.
    cmd_char: u8,
    /// Optional number of bytes to detect as part of the AT command.
    char_num: u8,
}

// 413
impl Default for AtCmdConfig {
    fn default() -> Self {
        Self {
            //pre_idle_count: None, // default is 0x901
            pre_idle_count: Some(0),
            //post_idle_count: None, // default is 0x901
            post_idle_count: Some(0),
            gap_timeout: None, // default is 11
            cmd_char: b'+',
            char_num: 1,
        }
    }
}

impl AtCmdConfig {
    pub fn with_cmd_char(mut self, cmd_char: u8) -> Self {
        self.cmd_char = cmd_char;
        self
    }
}

// 427
//struct UartBuilder<'d, Dm: DriverMode> {
struct UartBuilder<'d> {
    uart: AnyUart<'d>,
    //phantom: PhantomData<Dm>,
}

// 432
/*
impl<'d, Dm> UartBuilder<'d, Dm>
where
    Dm: DriverMode,
{
*/
impl<'d> UartBuilder<'d> {
    // 436
    fn new(uart: impl Instance + 'd) -> Self {
        Self {
            uart: uart.degrade(),
            //phantom: PhantomData,
        }
    }

    // 443
    //fn init(self, config: Config) -> Result<Uart<'d, Dm>, ConfigError> {}
    fn init(self, config: Config) -> Result<Uart<'d>, ConfigError> {
        let rx_guard = PeripheralGuard::new(self.uart.parts().0.peripheral);
        let tx_guard = PeripheralGuard::new(self.uart.parts().0.peripheral);

        let tx_pin = PinGuard::new_unconnected(/*self.uart.info().tx_signal*/);

        let mut serial = Uart {
            rx: UartRx {
                uart: unsafe { self.uart.clone_unchecked() },
                //phantom: PhantomData,
                _guard: rx_guard,
            },
            tx: UartTx {
                uart: self.uart,
                //phantom: PhantomData
                _guard: tx_guard,
                //rts_pin,
                tx_pin,
            },
        };
        serial.init(config)?;

        // we are using uart only async mode
        serial.set_async_interrupt_handler();

        Ok(serial)
    }
}

// 485
//pub struct Uart<'d, Dm: DriverMode> {}
pub struct Uart<'d> {
    //rx: UartRx<'d, Dm>,
    rx: UartRx<'d>,
    //tx: UartTx<'d, Dm>,
    tx: UartTx<'d>,
}

/// UART (Transmit)
// 492
//pub struct UartTx<'d, Dm: DriverMode> {
pub struct UartTx<'d> {
    uart: AnyUart<'d>,
    //phantom: PhantomData<Dm>,
    _guard: PeripheralGuard,
    //rts_pin: PinGuard,
    tx_pin: PinGuard,
}

/// UART (Receive)
// 502
//pub struct UartRx<'d, Dm: DriverMode> {
pub struct UartRx<'d> {
    uart: AnyUart<'d>,
    //phantom: PhantomData<Dm>,
    _guard: PeripheralGuard,
}

/// A configuration error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 512
pub enum ConfigError {
    /// The requested baud rate is not achievable.
    UnachievableBaudrate,

    /// The requested baud rate is not supported.
    ///
    /// This error is returned if:
    ///  * the baud rate exceeds 5MBaud or is equal to zero.
    ///  * the user has specified an exact baud rate or with some percentage of
    ///    deviation to the desired value, and the driver cannot reach this
    ///    speed.
    UnsupportedBaudrate,

    /// The requested  timeout exceeds the maximum value of 1023
    UnsupportedTimeout,

    /// The requested RX FIFO threshold exceeds the maximum value (127 bytes).
    UnsupportedRxFifoThreshold,

    /// The requested TX FIFO threshold exceeds the maximum value (127 bytes).
    UnsupportedTxFifoThreshold,
}

// 649
//impl<'d> UartTx<'d, Async>
impl<'d> UartTx<'d> {
    /// Write data into the TX buffer.
    ///
    /// This function writes the provided buffer `bytes` into the UART transmit
    /// buffer. If the buffer is full, the function waits asynchronously for
    /// space in the buffer to become available.
    ///
    /// The function returns the number of bytes written into the buffer. This
    /// may be less than the length of the buffer.
    ///
    /// Upon an error, the function returns immediately and the contents of the
    /// internal FIFO are not modified.
    ///
    /// ## Cancellation
    ///
    /// This function is cancellation safe.
    // 685
    pub async fn write_async(&mut self, bytes: &[u8]) -> Result<usize, TxError> {
        // We need to loop in case the TX empty interrupt was fired but not cleared
        // before, but the FIFO itself was filled up by a previous write.
        let space = loop {
            let tx_fifo_count = self.uart.info().tx_fifo_count();
            let space = Info::UART_FIFO_SIZE - tx_fifo_count;
            if space != 0 {
                break space;
            }
            // TxEvent::FiFoEmpty when the amount of data in Tx-FIFO is less than what txfifo_empty_thrhd specifies
            // when returns (Ready) future cleared all event(s) it waited for
            UartTxFuture::new(self.uart.reborrow(), TxEvent::FiFoEmpty).await;
        };

        let free = (space as usize).min(bytes.len());

        for &byte in &bytes[..free] {
            self.uart
                .info()
                .regs()
                .fifo()
                // writer - UART 0 accesses FIFO via this register.
                .write(|w| unsafe { w.rxfifo_rd_byte().bits(byte) });
        }

        Ok(free)
    }

    /// Asynchronously flushes the UART transmit buffer.
    ///
    /// This function ensures that all pending data in the transmit FIFO has
    /// been sent over the UART. If the FIFO contains data, it waits for the
    /// transmission to complete before returning.
    ///
    /// ## Cancellation
    ///
    /// This function is cancellation safe.
    // 719
    pub async fn flush_async(&mut self) -> Result<(), TxError> {
        // Nothing is guaranteed to clear the Done status, so let's loop here in case Tx
        // was Done before the last write operation that pushed data into the
        // FIFO.
        while self.uart.info().tx_fifo_count() > 0 {
            // TxEvent::Done when transmitter has send out all data in FIFO
            UartTxFuture::new(self.uart.reborrow(), TxEvent::Done).await;
        }

        self.flush_last_byte();

        Ok(())
    }
}

// 733
/*
impl<'d, Dm> UartTx<'d, Dm>
where
    Dm: DriverMode,
*/
impl<'d> UartTx<'d> {
    /// Assign the TX pin for UART instance.
    ///
    /// Sets the specified pin to push-pull output and connects it to the UART
    /// TX signal.
    ///
    /// Disconnects the previous pin that was assigned with `with_tx`.
    // 757
    pub fn with_tx(mut self, tx: impl PeripheralOutput<'d>) -> Self {
        let tx = tx.into();

        // Make sure we don't cause an unexpected low pulse on the pin.
        tx.set_output_high(true);
        tx.apply_output_config(&OutputConfig::default());
        tx.set_output_enable(true);

        self.tx_pin = tx.connect_with_guard(self.uart.info().tx_signal);

        self
    }

    /// Change the configuration.
    ///
    /// ## Errors
    ///
    /// This function returns a [`ConfigError`] if the configuration is not
    /// supported by the hardware.
    // 777
    pub fn apply_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.uart
            .info()
            .set_tx_fifo_empty_threshold(config.tx.fifo_empty_threshold)?;
        self.uart.info().txfifo_reset();
        Ok(())
    }

    /// Write bytes.
    ///
    /// This function writes data to the internal TX FIFO of the UART
    /// peripheral. The data is then transmitted over the UART TX line.
    ///
    /// The function returns the number of bytes written to the FIFO. This may
    /// be less than the length of the provided data. The function may only
    /// return 0 if the provided data is empty.
    ///
    /// ## Errors
    ///
    /// This function returns a [`TxError`] if an error occurred during the
    /// write operation.
    // 812
    pub fn write(&mut self, data: &[u8]) -> Result<usize, TxError> {
        self.uart.info().write(data)
    }

    // 830
    fn flush_last_byte(&mut self) {
        // This function handles an edge case that happens when the TX FIFO count
        // changes to 0. The FSM is in the Idle state for a short while after
        // the last byte is moved out of the FIFO. It is unclear how long this
        // takes, but 10us seems to be a good enough duration to wait, for both
        // fast and slow baud rates.
        crate::rom::ets_delay_us(10);
        while !self.is_tx_idle() {}
    }

    /// Checks if the TX line is idle for this UART instance.
    ///
    /// Returns `true` if the transmit line is idle, meaning no data is
    /// currently being transmitted.
    // 844
    fn is_tx_idle(&self) -> bool {
        let status = self.regs().fsm_status();

        status.read().st_utx_out().bits() == 0x0
    }

    /// Disables all TX-related interrupts for this UART instance.
    ///
    /// This function clears and disables the `transmit FIFO empty` interrupt,
    /// `transmit break done`, `transmit break idle done`, and `transmit done`
    /// interrupts.
    // 858
    fn disable_tx_interrupts(&self) {
        self.regs().int_clr().write(|w| {
            w.txfifo_empty().clear_bit_by_one();
            w.tx_brk_done().clear_bit_by_one();
            w.tx_brk_idle_done().clear_bit_by_one();
            w.tx_done().clear_bit_by_one()
        });

        self.regs().int_ena().write(|w| {
            w.txfifo_empty().clear_bit();
            w.tx_brk_done().clear_bit();
            w.tx_brk_idle_done().clear_bit();
            w.tx_done().clear_bit()
        });
    }

    // 874
    fn regs(&self) -> &RegisterBlock {
        self.uart.info().regs()
    }
}

#[inline(always)]
// 880
fn sync_regs(_register_block: &RegisterBlock) {
    let update_reg = _register_block.id();

    // Software write 1 would synchronize registers into UART Core clock domain
    update_reg.modify(|_, w| w.reg_update().set_bit());

    // and would be cleared by hardware after synchronization is done.
    while update_reg.read().reg_update().bit_is_set() {
        // wait
    }
}

//impl<'d> UartRx<'d, Async>
// 942
impl<'d> UartRx<'d> {
    // 961
    pub async fn wait_for_buffered_data(
        &mut self,
        minimum: usize,
        max_threshold: usize,
        listen_for_timeout: bool,
    ) -> Result<(), RxError> {
        let current_threshold = self.uart.info().rx_fifo_full_threshold();

        // User preference takes priority.
        let max_threshold = max_threshold.min(current_threshold as usize) as u16;
        let minimum = minimum.min(Info::RX_FIFO_MAX_THRHD as usize) as u16;

        // The effective threshold must be >= minimum. We ensure this by lowering the minimum number
        // of returnable bytes.
        let minimum = minimum.min(max_threshold);

        if self.uart.info().rx_fifo_count() < minimum {
            // We're ignoring the user configuration here to ensure that this is not waiting
            // for more data than the buffer. We'll restore the original value after the
            // future resolved.
            let info = self.uart.info();
            unwrap!(info.set_rx_fifo_full_threshold(max_threshold));
            let _guard = OnDrop::new(|| {
                unwrap!(info.set_rx_fifo_full_threshold(current_threshold));
            });

            // Wait for space or event
            let mut events = RxEvent::FifoFull
                | RxEvent::FifoOvf
                | RxEvent::FrameError
                | RxEvent::GlitchDetected
                | RxEvent::ParityError;

            if self.regs().at_cmd_char().read().char_num().bits() > 0 {
                events |= RxEvent::CmdCharDetected;
            }

            let reg_en = self.regs().conf1();
            // default is 10 bits = 1 character
            if listen_for_timeout && reg_en.read().rx_tout_en().bit_is_set() {
                events |= RxEvent::FifoTout;
            }

            let events = UartRxFuture::new(self.uart.reborrow(), events).await;
            info!("##### UartRxFuture returned with events: {}", events);

            // ignore FifoFull, CmdCharDetected and FifoTout, all others are error
            let result = rx_event_check_for_error(events);
            if let Err(error) = result {
                if error == RxError::FifoOverflowed {
                    self.uart.info().rxfifo_reset();
                }
                return Err(error);
            }
        }

        // either there are enough data (more than 'minimum') rigth away in the fifo
        // or we wait and get irq when there are 'prefered' number of bytes in the fifo
        // or we wait and we get irq of CmdCharDetected or FifoTout
        Ok(())
    }

    /// Read data asynchronously.
    ///
    /// This function reads data from the UART receive buffer into the
    /// provided buffer. If the buffer is empty, the function waits
    /// asynchronously for data to become available, or for an error to occur.
    ///
    /// The function returns the number of bytes read into the buffer. This may
    /// be less than the length of the buffer.
    ///
    /// Note that this function may ignore the `rx_fifo_full_threshold` setting
    /// to ensure that it does not wait for more data than the buffer can hold.
    ///
    /// Upon an error, the function returns immediately and the contents of the
    /// internal FIFO are not modified.
    ///
    /// ## Cancellation
    ///
    /// This function is cancellation safe.
    // 1041
    pub async fn read_async(&mut self, buf: &mut [u8], timeout: bool) -> Result<usize, RxError> {
        if buf.is_empty() {
            return Ok(0);
        }

        // either there are one or more byte right away in the fifo
        // or we are waiting for buf.len() number of data arriving into fifo
        // or other interrupt (e.g CmdCharDetected or FifoTout or some of the four types of error) occured
        self.wait_for_buffered_data(1, buf.len(), timeout).await?;

        // there is data (based on the rules above), read them
        self.uart.info().read_buffered(buf)
    }

    /// Fill buffer asynchronously.
    ///
    /// This function reads data into the provided buffer. If the internal FIFO
    /// does not contain enough data, the function waits asynchronously for data
    /// to become available, or for an error to occur.
    ///
    /// Note that this function may ignore the `rx_fifo_full_threshold` setting
    /// to ensure that it does not wait for more data than the buffer can hold.
    ///
    /// ## Cancellation
    ///
    /// This function is **not** cancellation safe. If the future is dropped
    /// before it resolves, or if an error occurs during the read operation,
    /// previously read data may be lost.
    // 1065
    pub async fn read_exact_async(&mut self, mut buf: &mut [u8]) -> Result<(), RxError> {
        if buf.is_empty() {
            return Ok(());
        }

        // Drain the buffer first, there's no point in waiting for data we've already received.
        let read = self.uart.info().read_buffered(buf)?;
        buf = &mut buf[read..];

        while !buf.is_empty() {
            // No point in listening for timeouts, as we're waiting for an exact amount of
            // data. On ESP32 and S2, the timeout interrupt can't be cleared unless the FIFO
            // is empty, so listening could cause an infinite loop here.
            self.wait_for_buffered_data(buf.len(), buf.len(), false)
                .await?;

            let read = self.uart.info().read_buffered(buf)?;
            buf = &mut buf[read..];
        }

        Ok(())
    }
}

// 1081
/*
impl<'d, Dm> UartRx<'d, Dm>
where
    Dm: DriverMode,
*/
impl<'d> UartRx<'d> {
    // 1085
    fn regs(&self) -> &RegisterBlock {
        self.uart.info().regs()
    }

    /// Assign the RX pin for UART instance.
    ///
    /// Sets the specified pin to input and connects it to the UART RX signal.
    ///
    /// Note: when you listen for the output of the UART peripheral, you should
    /// configure the driver side (i.e. the TX pin), or ensure that the line is
    /// initially high, to avoid receiving a non-data byte caused by an
    /// initial low signal level.
    // 1113
    pub fn with_rx(self, rx: impl PeripheralInput<'d>) -> Self {
        let rx = rx.into();

        rx.apply_input_config(&InputConfig::default().with_pull(Pull::Up));
        rx.set_input_enable(true);

        self.uart.info().rx_signal.connect_to(&rx);

        self
    }

    /// Change the configuration.
    ///
    /// ## Errors
    ///
    /// This function returns a [`ConfigError`] if the configuration is not
    /// supported by the hardware.
    // 1131
    pub fn apply_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.uart
            .info()
            .set_rx_fifo_full_threshold(config.rx.fifo_full_threshold)?;
        self.uart
            .info()
            .set_rx_timeout(config.rx.timeout, self.uart.info().current_symbol_length())?;

        self.uart.info().rxfifo_reset();
        Ok(())
    }

    /// Disables all RX-related interrupts for this UART instance.
    ///
    /// This function clears and disables the `receive FIFO full` interrupt,
    /// `receive FIFO overflow`, `receive FIFO timeout`, and `AT command
    /// byte detection` interrupts.
    // 1209
    fn disable_rx_interrupts(&self) {
        self.regs().int_clr().write(|w| {
            w.rxfifo_full().clear_bit_by_one();
            w.rxfifo_ovf().clear_bit_by_one();
            w.rxfifo_tout().clear_bit_by_one();
            w.at_cmd_char_det().clear_bit_by_one()
        });

        self.regs().int_ena().write(|w| {
            w.rxfifo_full().clear_bit();
            w.rxfifo_ovf().clear_bit();
            w.rxfifo_tout().clear_bit();
            w.at_cmd_char_det().clear_bit()
        });
    }
}

// 1226
//impl<'d> Uart<'d, Blocking>
impl<'d> Uart<'d> {
    /// Create a new UART instance in [`Blocking`] mode.
    ///
    /// ## Errors
    ///
    /// This function returns a [`ConfigError`] if the configuration is not
    /// supported by the hardware.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # use esp_hal::uart::{Config, Uart};
    /// let mut uart1 = Uart::new(
    ///     peripherals.UART1,
    ///     Config::default())?
    /// .with_rx(peripherals.GPIO1)
    /// .with_tx(peripherals.GPIO2);
    /// # Ok(())
    /// # }
    /// ```
    // 1247
    //pub fn new(uart: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
    pub fn new(uart: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
        UartBuilder::new(uart).init(config)
    }
}

/// List of exposed UART events.
// should be modified parallel to Info::enable_listen()
#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 1456
pub enum UartInterrupt {
    /// Indicates that the received has detected the configured
    /// [`Uart::set_at_cmd`] byte.
    AtCmd,

    /// The transmitter has finished sending out all data from the FIFO.
    TxDone,

    /// The receiver has received more data than what
    /// [`RxConfig::fifo_full_threshold`] specifies.
    RxFifoFull,

    /// The receiver has not received any data for the time
    /// [`RxConfig::with_timeout`] specifies.
    RxTimeout,
}

/*
impl<'d, Dm> Uart<'d, Dm>
where
    Dm: DriverMode,
*/
// 1473
impl<'d> Uart<'d> {
    /// Assign the RX pin for UART instance.
    ///
    /// Sets the specified pin to input and connects it to the UART RX signal.
    ///
    /// Note: when you listen for the output of the UART peripheral, you should
    /// configure the driver side (i.e. the TX pin), or ensure that the line is
    /// initially high, to avoid receiving a non-data byte caused by an
    /// initial low signal level.
    // 1485
    pub fn with_rx(mut self, rx: impl PeripheralInput<'d>) -> Self {
        self.rx = self.rx.with_rx(rx);
        self
    }

    /// Assign the TX pin for UART instance.
    ///
    /// Sets the specified pin to push-pull output and connects it to the UART
    /// TX signal.
    // 1494
    pub fn with_tx(mut self, tx: impl PeripheralOutput<'d>) -> Self {
        self.tx = self.tx.with_tx(tx);
        self
    }

    // 1511
    fn regs(&self) -> &RegisterBlock {
        // `self.tx.uart` and `self.rx.uart` are the same
        self.tx.uart.info().regs()
    }

    /// Change the configuration.
    ///
    /// ## Errors
    ///
    /// This function returns a [`ConfigError`] if the configuration is not
    /// supported by the hardware.
    // 1595
    pub fn apply_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        // Must apply the common settings first, as `rx.apply_config` reads back symbol
        // size.
        self.rx.uart.info().apply_config(config)?;
        self.rx.apply_config(config)?;
        self.tx.apply_config(config)?;
        Ok(())
    }

    /// Split the UART into a transmitter and receiver
    ///
    /// This is particularly useful when having two tasks correlating to
    /// transmitting and receiving.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// # use esp_hal::uart::{Config, Uart};
    /// # let mut uart1 = Uart::new(
    /// #     peripherals.UART1,
    /// #     Config::default())?
    /// # .with_rx(peripherals.GPIO1)
    /// # .with_tx(peripherals.GPIO2);
    /// // The UART can be split into separate Transmit and Receive components:
    /// let (mut rx, mut tx) = uart1.split();
    ///
    /// // Each component can be used individually to interact with the UART:
    /// tx.write(&[42u8])?;
    /// let mut byte = [0u8; 1];
    /// rx.read(&mut byte);
    /// # Ok(())
    /// # }
    /// ```
    //pub fn split(self) -> (UartRx<'d, Dm>, UartTx<'d, Dm>) {
    // 1630
    pub fn split(self) -> (UartRx<'d>, UartTx<'d>) {
        (self.rx, self.tx)
    }

    /// Configures the AT-CMD detection settings
    // 1663
    pub fn set_at_cmd(&mut self, config: AtCmdConfig) {
        // CLEAR this bit to disable UART TX/RX clock
        self.regs()
            .clk_conf()
            .modify(|_, w| w.sclk_en().clear_bit());

        self.regs().at_cmd_char().write(|w| unsafe {
            w.at_cmd_char().bits(config.cmd_char);
            w.char_num().bits(config.char_num)
        });

        if let Some(pre_idle_count) = config.pre_idle_count {
            self.regs()
                .at_cmd_precnt()
                .write(|w| unsafe { w.pre_idle_num().bits(pre_idle_count as _) });
        }

        if let Some(post_idle_count) = config.post_idle_count {
            self.regs()
                .at_cmd_postcnt()
                .write(|w| unsafe { w.post_idle_num().bits(post_idle_count as _) });
        }

        if let Some(gap_timeout) = config.gap_timeout {
            self.regs()
                .at_cmd_gaptout()
                .write(|w| unsafe { w.rx_gap_tout().bits(gap_timeout as _) });
        }

        // Set this bit to enable UART TX/RX clock
        self.regs().clk_conf().modify(|_, w| w.sclk_en().set_bit());

        sync_regs(self.regs());
    }

    #[inline(always)]
    // 1699
    fn init(&mut self, config: Config) -> Result<(), ConfigError> {
        /*
        crate::peripherals::SYSTEM::regs()
            // enable the clock for UART RAM
            .perip_clk_en0()
            .modify(|_, w| w.uart_mem_clk_en().set_bit());

        self.uart_peripheral_reset();
        */

        self.rx.disable_rx_interrupts();
        self.tx.disable_tx_interrupts();

        self.apply_config(&config)?;

        // Reset Tx/Rx FIFOs
        self.rx.uart.info().rxfifo_reset();
        self.rx.uart.info().txfifo_reset();

        // Don't wait after transmissions by default,
        // so that bytes written to TX FIFO are always immediately transmitted.
        self.regs()
            .idle_conf()
            // configure the duration time between transfers (n the unit of bit time )
            .modify(|_, w| unsafe { w.tx_idle_num().bits(0) });

        // Setting err_wr_mask stops uart from storing data when data is wrong according
        // to reference manual
        self.regs().conf0().modify(|_, w| w.err_wr_mask().set_bit());

        crate::rom::ets_delay_us(15);

        // Make sure we are starting in a "clean state" - previous operations might have
        // run into error conditions
        // clear all pending interrupt
        self.regs().int_clr().write(|w| unsafe { w.bits(u32::MAX) });

        Ok(())
    }

    fn set_async_interrupt_handler(&mut self) {
        // `self.tx.uart` and `self.rx.uart` are the same
        self.tx.uart.info().set_async_interrupt_handler();
    }

    // 1744
    /*
    #[cfg(uart0)]
    fn is_instance(&self, other: impl Instance) -> bool {
        self.tx.uart.info().is_instance(other)
    }

    #[inline(always)]
    // 1749
    fn uart_peripheral_reset(&self) {
        // don't reset the console UART - this will cause trouble (i.e. the UART will
        // start to transmit garbage)
        //
        // We should only reset the console UART if it was absolutely unused before.
        // Apparently the bootloader (and maybe the ROM code) writing to the UART is
        // already enough to make this a no-go. (i.e. one needs to mute the ROM
        // code via efuse / strapping pin AND use a silent bootloader)
        //
        // TODO: make this configurable
        // see https://github.com/espressif/esp-idf/blob/5f4249357372f209fdd57288265741aaba21a2b1/components/esp_driver_uart/src/uart.c#L179
        // I should check it after installation whether it is a real problem
        // since we use SERIAL_JTAG for console
        /*
        #[cfg(uart0)]
        if self.is_instance(unsafe { crate::peripherals::UART0::steal() }) {
            return;
        }
        */

        fn rst_core(_reg_block: &RegisterBlock, _enable: bool) {
            #[cfg(not(any(esp32, esp32s2, esp32c6, esp32h2)))]
            _reg_block
                // Write 1 and then write 0 to this bit, to reset UART TX/RX
                .clk_conf()
                .modify(|_, w| w.rst_core().bit(_enable));
        }

        rst_core(self.regs(), true);
        PeripheralClockControl::reset(self.tx.uart.info().peripheral);
        rst_core(self.regs(), false);
    }
    */
}

#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1988
pub(crate) enum TxEvent {
    Done,
    FiFoEmpty,
}

#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1996
pub(crate) enum RxEvent {
    FifoFull,
    CmdCharDetected,
    FifoOvf,
    FifoTout,
    GlitchDetected,
    FrameError,
    ParityError,
}

// 2006
fn rx_event_check_for_error(events: EnumSet<RxEvent>) -> Result<(), RxError> {
    for event in events {
        match event {
            RxEvent::FifoOvf => return Err(RxError::FifoOverflowed),
            RxEvent::GlitchDetected => return Err(RxError::GlitchOccurred),
            RxEvent::FrameError => return Err(RxError::FrameFormatViolated),
            RxEvent::ParityError => return Err(RxError::ParityMismatch),
            RxEvent::FifoFull | RxEvent::CmdCharDetected | RxEvent::FifoTout => continue,
        }
    }

    Ok(())
}

/// A future that resolves when the passed interrupt is triggered,
/// or has been triggered in the meantime (flag set in INT_RAW).
/// Upon construction the future enables the passed interrupt and when it
/// is dropped it disables the interrupt again. The future returns the event
/// that was initially passed, when it resolves.
// 2026
struct UartRxFuture {
    events: EnumSet<RxEvent>,
    uart: &'static Info,
    state: &'static State,
    registered: bool,
}

// 2033
impl UartRxFuture {
    fn new(uart: impl Instance, events: impl Into<EnumSet<RxEvent>>) -> Self {
        Self {
            events: events.into(),
            uart: uart.info(),
            state: uart.state(),
            registered: false,
        }
    }
}

// 2044
impl core::future::Future for UartRxFuture {
    type Output = EnumSet<RxEvent>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let events = self.uart.rx_events().intersection(self.events);
        if !events.is_empty() {
            self.uart.clear_rx_events(events);
            Poll::Ready(events)
        } else {
            self.state.rx_waker.register(cx.waker());
            if !self.registered {
                self.uart.enable_listen_rx(self.events, true);
                self.registered = true;
            }
            Poll::Pending
        }
    }
}

impl Drop for UartRxFuture {
    fn drop(&mut self) {
        // Although the isr disables the interrupt that occurred directly, we need to
        // disable the other interrupts (= the ones that did not occur), as
        // soon as this future goes out of scope.
        self.uart.enable_listen_rx(self.events, false);
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
// 2076
struct UartTxFuture {
    events: EnumSet<TxEvent>,
    uart: &'static Info,
    state: &'static State,
    registered: bool,
}

// 2083
impl UartTxFuture {
    fn new(uart: impl Instance, events: impl Into<EnumSet<TxEvent>>) -> Self {
        Self {
            events: events.into(),
            uart: uart.info(),
            state: uart.state(),
            registered: false,
        }
    }
}

// 2094
impl core::future::Future for UartTxFuture {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let events = self.uart.tx_events().intersection(self.events);
        if !events.is_empty() {
            self.uart.clear_tx_events(events);
            Poll::Ready(())
        } else {
            self.state.tx_waker.register(cx.waker());
            if !self.registered {
                self.uart.enable_listen_tx(self.events, true);
                self.registered = true;
            }
            Poll::Pending
        }
    }
}

// 2116
impl Drop for UartTxFuture {
    fn drop(&mut self) {
        // Although the isr disables the interrupt that occurred directly, we need to
        // disable the other interrupts (= the ones that did not occur), as
        // soon as this future goes out of scope.
        self.uart.enable_listen_tx(self.events, false);
    }
}

/// Interrupt handler for all UART instances
/// Clears and disables interrupts that have occurred and have their enable
/// bit set. The fact that an interrupt has been disabled is used by the
/// futures to detect that they should indeed resolve after being woken up
// 2177
pub(super) fn intr_handler(uart: &Info, state: &State) {
    let interrupts = uart.regs().int_st().read();
    let interrupt_bits = interrupts.bits(); // = int_raw & int_ena

    // This is the status bit for rxfifo_full_int_raw when rxfifo_full_int_ena is set to 1
    let rx_wake = interrupts.rxfifo_full().bit_is_set()
        // This is the status bit for rxfifo_ovf_int_raw when rxfifo_ovf_int_ena is set to 1
        | interrupts.rxfifo_ovf().bit_is_set()
        // This is the status bit for rxfifo_tout_int_raw when rxfifo_tout_int_ena is set to 1
        | interrupts.rxfifo_tout().bit_is_set()
        // This is the status bit for at_cmd_det_int_raw when at_cmd_char_det_int_ena is set to 1
        | interrupts.at_cmd_char_det().bit_is_set()
        // This is the status bit for glitch_det_int_raw when glitch_det_int_ena is set to 1
        | interrupts.glitch_det().bit_is_set()
        // This is the status bit for frm_err_int_raw when frm_err_int_ena is set to 1
        | interrupts.frm_err().bit_is_set()
        // This is the status bit for parity_err_int_raw when parity_err_int_ena is set to 1
        | interrupts.parity_err().bit_is_set();
    // This is the status bit for tx_done_int_raw when tx_done_int_ena is set to 1
    // This is the status bit for txfifo_empty_int_raw when txfifo_empty_int_ena is set to 1
    let tx_wake = interrupts.tx_done().bit_is_set() | interrupts.txfifo_empty().bit_is_set();

    uart.regs()
        .int_ena()
        .modify(|r, w| unsafe { w.bits(r.bits() & !interrupt_bits) });

    if tx_wake {
        state.tx_waker.wake();
    }
    if rx_wake {
        state.rx_waker.wake();
    }
}

// 2427
/// A peripheral singleton compatible with the UART driver.
//pub trait Instance: crate::private::Sealed + IntoAnyUart
//pub trait Instance: IntoAnyUart {
pub trait Instance: any::Degrade {
    /// Returns the peripheral data and state describing this UART instance.
    // 2428
    fn parts(&self) -> (&'static Info, &'static State);

    /// Returns the peripheral data describing this UART instance.
    #[inline(always)]
    // 2433
    fn info(&self) -> &'static Info {
        self.parts().0
    }

    /// Returns the peripheral state for this UART instance.
    #[inline(always)]
    // 2440
    fn state(&self) -> &'static State {
        self.parts().1
    }
}

/// Peripheral data describing a particular UART instance.
#[non_exhaustive]
// 2448
pub struct Info {
    /// Pointer to the register block for this UART instance.
    ///
    /// Use [Self::register_block] to access the register block.
    pub register_block: *const RegisterBlock,

    /// The system peripheral marker.
    pub peripheral: crate::system::Peripheral,

    /// Interrupt handler for the asynchronous operations of this UART instance.
    pub async_handler: InterruptHandler,

    /// Interrupt for this UART instance.
    pub interrupt: Interrupt,

    /// TX pin
    pub tx_signal: OutputSignal,

    /// RX pin
    pub rx_signal: InputSignal,
}

/// Peripheral state for a UART instance.
#[non_exhaustive]
// 2479
pub struct State {
    /// Waker for the asynchronous RX operations.
    pub rx_waker: AtomicWaker,

    /// Waker for the asynchronous TX operations.
    pub tx_waker: AtomicWaker,
    /*
    /// Stores whether the TX half is configured for async operation.
    pub is_rx_async: AtomicBool,

    /// Stores whether the RX half is configured for async operation.
    pub is_tx_async: AtomicBool,
    */
}

// 2493
impl Info {
    // Currently we don't support merging adjacent FIFO memory, so the max size is
    // 128 bytes, the max threshold is 127 bytes.
    //const UART_FIFO_SIZE: u16 = 128;
    const UART_FIFO_SIZE: u16 = property!("uart.ram_size");
    //const RX_FIFO_MAX_THRHD: u16 = 127;
    const RX_FIFO_MAX_THRHD: u16 = Self::UART_FIFO_SIZE - 1;
    const TX_FIFO_MAX_THRHD: u16 = Self::RX_FIFO_MAX_THRHD;

    /// Returns the register block for this UART instance.
    // 2501
    pub fn regs(&self) -> &RegisterBlock {
        unsafe { &*self.register_block }
    }

    /// Listen for the given interrupts
    // this should be modified if you are interested other uart interrupts/events
    // since enable_listen_tx/rx, tx/rx_events, clear_tx/rx_events are handling all
    // it should be modified parallel to enum UartInterrupt
    // 2506
    fn enable_listen(&self, interrupts: EnumSet<UartInterrupt>, enable: bool) {
        let reg_block = self.regs();

        reg_block.int_ena().modify(|_, w| {
            for interrupt in interrupts {
                match interrupt {
                    UartInterrupt::AtCmd => w.at_cmd_char_det().bit(enable),
                    UartInterrupt::TxDone => w.tx_done().bit(enable),
                    UartInterrupt::RxFifoFull => w.rxfifo_full().bit(enable),
                    UartInterrupt::RxTimeout => w.rxfifo_tout().bit(enable),
                };
            }
            w
        });
    }

    // 2544
    fn clear_interrupts(&self, interrupts: EnumSet<UartInterrupt>) {
        let reg_block = self.regs();

        reg_block.int_clr().write(|w| {
            for interrupt in interrupts {
                match interrupt {
                    UartInterrupt::AtCmd => w.at_cmd_char_det().clear_bit_by_one(),
                    UartInterrupt::TxDone => w.tx_done().clear_bit_by_one(),
                    UartInterrupt::RxFifoFull => w.rxfifo_full().clear_bit_by_one(),
                    UartInterrupt::RxTimeout => w.rxfifo_tout().clear_bit_by_one(),
                };
            }
            w
        });
    }

    // 2560
    fn set_async_interrupt_handler(&self) {
        crate::interrupt::disable(Cpu::ProCpu, self.interrupt);

        self.enable_listen(EnumSet::all(), false);
        self.clear_interrupts(EnumSet::all());
        unsafe { crate::interrupt::bind_interrupt(self.interrupt, self.async_handler.handler()) };

        unwrap!(crate::interrupt::enable(
            self.interrupt,
            // since #[handler] macro does not define priority
            // default min will be set
            self.async_handler.priority()
        ));
    }

    // 2574
    fn apply_config(&self, config: &Config) -> Result<(), ConfigError> {
        config.validate()?;
        self.change_baud(config)?;
        self.change_data_bits(config.data_bits);
        self.change_parity(config.parity);
        self.change_stop_bits(config.stop_bits);
        //self.change_flow_control(config.sw_flow_ctrl, config.hw_flow_ctrl);

        Ok(())
    }

    // 2585
    fn enable_listen_tx(&self, events: EnumSet<TxEvent>, enable: bool) {
        self.regs().int_ena().modify(|_, w| {
            for event in events {
                match event {
                    TxEvent::Done => w.tx_done().bit(enable),
                    TxEvent::FiFoEmpty => w.txfifo_empty().bit(enable),
                };
            }
            w
        });
    }

    // 2597
    fn tx_events(&self) -> EnumSet<TxEvent> {
        let pending_interrupts = self.regs().int_raw().read();
        let mut active_events = EnumSet::new();

        // This interrupt raw bit turns to high level when transmitter has send out all data in FIFO.
        if pending_interrupts.tx_done().bit_is_set() {
            active_events |= TxEvent::Done;
        }
        // This interrupt raw bit turns to high level when the amount of data
        // in Tx-FIFO is less than what txfifo_empty_thrhd specifies
        if pending_interrupts.txfifo_empty().bit_is_set() {
            active_events |= TxEvent::FiFoEmpty;
        }

        active_events
    }

    // 2611
    fn clear_tx_events(&self, events: impl Into<EnumSet<TxEvent>>) {
        let events = events.into();
        self.regs().int_clr().write(|w| {
            for event in events {
                match event {
                    TxEvent::FiFoEmpty => w.txfifo_empty().clear_bit_by_one(),
                    TxEvent::Done => w.tx_done().clear_bit_by_one(),
                };
            }
            w
        });
    }

    // 2624
    fn enable_listen_rx(&self, events: EnumSet<RxEvent>, enable: bool) {
        self.regs().int_ena().modify(|_, w| {
            for event in events {
                match event {
                    RxEvent::FifoFull => w.rxfifo_full().bit(enable),
                    RxEvent::CmdCharDetected => w.at_cmd_char_det().bit(enable),

                    RxEvent::FifoOvf => w.rxfifo_ovf().bit(enable),
                    RxEvent::FifoTout => w.rxfifo_tout().bit(enable),
                    RxEvent::GlitchDetected => w.glitch_det().bit(enable),
                    RxEvent::FrameError => w.frm_err().bit(enable),
                    RxEvent::ParityError => w.parity_err().bit(enable),
                };
            }
            w
        });
    }

    // 2642
    fn rx_events(&self) -> EnumSet<RxEvent> {
        let pending_interrupts = self.regs().int_raw().read();
        let mut active_events = EnumSet::new();

        if pending_interrupts.rxfifo_full().bit_is_set() {
            active_events |= RxEvent::FifoFull;
        }
        if pending_interrupts.at_cmd_char_det().bit_is_set() {
            active_events |= RxEvent::CmdCharDetected;
        }
        if pending_interrupts.rxfifo_ovf().bit_is_set() {
            active_events |= RxEvent::FifoOvf;
        }
        if pending_interrupts.rxfifo_tout().bit_is_set() {
            active_events |= RxEvent::FifoTout;
        }
        if pending_interrupts.glitch_det().bit_is_set() {
            active_events |= RxEvent::GlitchDetected;
        }
        if pending_interrupts.frm_err().bit_is_set() {
            active_events |= RxEvent::FrameError;
        }
        if pending_interrupts.parity_err().bit_is_set() {
            active_events |= RxEvent::ParityError;
        }

        active_events
    }

    // 2671
    fn clear_rx_events(&self, events: impl Into<EnumSet<RxEvent>>) {
        let events = events.into();
        self.regs().int_clr().write(|w| {
            for event in events {
                match event {
                    RxEvent::FifoFull => w.rxfifo_full().clear_bit_by_one(),
                    RxEvent::CmdCharDetected => w.at_cmd_char_det().clear_bit_by_one(),

                    RxEvent::FifoOvf => w.rxfifo_ovf().clear_bit_by_one(),
                    RxEvent::FifoTout => w.rxfifo_tout().clear_bit_by_one(),
                    RxEvent::GlitchDetected => w.glitch_det().clear_bit_by_one(),
                    RxEvent::FrameError => w.frm_err().clear_bit_by_one(),
                    RxEvent::ParityError => w.parity_err().clear_bit_by_one(),
                };
            }
            w
        });
    }

    /// Configures the RX-FIFO threshold
    ///
    /// ## Errors
    ///
    /// [ConfigError::UnsupportedRxFifoThreshold] if the provided value exceeds
    /// [`Info::RX_FIFO_MAX_THRHD`].
    // 2696
    fn set_rx_fifo_full_threshold(&self, threshold: u16) -> Result<(), ConfigError> {
        if threshold > Self::RX_FIFO_MAX_THRHD {
            return Err(ConfigError::UnsupportedRxFifoThreshold);
        }

        self.regs()
            .conf1()
            // It will produce rxfifo_full_int interrupt
            // when receiver receives more data than this register value.
            .modify(|_, w| unsafe { w.rxfifo_full_thrhd().bits(threshold as _) });

        Ok(())
    }

    /// Reads the RX-FIFO threshold
    // 2710
    #[allow(clippy::useless_conversion)]
    fn rx_fifo_full_threshold(&self) -> u16 {
        self.regs().conf1().read().rxfifo_full_thrhd().bits().into()
    }

    /// Configures the TX-FIFO threshold
    ///
    /// ## Errors
    ///
    /// [ConfigError::UnsupportedTxFifoThreshold] if the provided value exceeds
    /// [`Info::TX_FIFO_MAX_THRHD`].
    // 2720
    fn set_tx_fifo_empty_threshold(&self, threshold: u16) -> Result<(), ConfigError> {
        if threshold > Self::TX_FIFO_MAX_THRHD {
            return Err(ConfigError::UnsupportedTxFifoThreshold);
        }

        self.regs()
            .conf1()
            .modify(|_, w| unsafe { w.txfifo_empty_thrhd().bits(threshold as _) });

        Ok(())
    }

    /// Configures the Receive Timeout detection setting
    ///
    /// ## Arguments
    ///
    /// `timeout` - the number of symbols ("bytes") to wait for before
    /// triggering a timeout. Pass None to disable the timeout.
    ///
    /// ## Errors
    ///
    /// [ConfigError::UnsupportedTimeout] if the provided value exceeds
    /// the maximum value for SOC:
    /// - `esp32`: Symbol size is fixed to 8, do not pass a value > **0x7F**.
    /// - `esp32c2`, `esp32c3`, `esp32c6`, `esp32h2`, esp32s2`, esp32s3`: The
    ///   value you pass times the symbol size must be <= **0x3FF**
    // 2746
    fn set_rx_timeout(&self, timeout: Option<u8>, _symbol_len: u8) -> Result<(), ConfigError> {
        const MAX_THRHD: u16 = 0x3FF; // 10 bits

        let register_block = self.regs();

        if let Some(timeout) = timeout {
            let timeout_reg = timeout as u16 * _symbol_len as u16;

            if timeout_reg > MAX_THRHD {
                return Err(ConfigError::UnsupportedTimeout);
            }

            let reg_thrhd = register_block.mem_conf();
            // This register is used to configure the threshold time that
            // receiver takes to receive one byte. The rxfifo_tout_int interrupt will be trigger
            // when the receiver takes more time to receive one byte with rx_tout_en set to 1.
            // default is 10 bits = 1 character
            reg_thrhd.modify(|_, w| unsafe { w.rx_tout_thrhd().bits(timeout_reg) });
        }

        let reg_en = register_block.conf1();
        // This is the enble bit for uart receiver's timeout function.
        reg_en.modify(|_, w| w.rx_tout_en().bit(timeout.is_some()));

        self.sync_regs();

        Ok(())
    }

    // 2795
    /*
    #[cfg(uart0)]
    fn is_instance(&self, other: impl Instance) -> bool {
        self == other.info()
    */

    // 2799
    fn sync_regs(&self) {
        sync_regs(self.regs());
    }

    // 2803
    fn change_baud(&self, config: &Config) -> Result<(), ConfigError> {
        let clocks = Clocks::get();
        let clk = match config.clock_source {
            ClockSource::Apb => clocks.apb_clock.as_hz(),
            ClockSource::RcFast => crate::soc::constants::RC_FAST_CLK.as_hz(),
        };

        const MAX_DIV: u32 = 0b1111_1111_1111 - 1;
        let clk_div = (clk.div_ceil(MAX_DIV)).div_ceil(config.baudrate);

        if matches!(config.clock_source, ClockSource::RcFast) {
            // we are on the dafult ClockSource::Apb path
            /*
            crate::peripherals::LPWR::regs()
                .clk_conf()
                .modify(|_, w| w.dig_clk8m_en().variant(true));
            // small delay whilst the clock source changes (SOC_DELAY_RC_FAST_DIGI_SWITCH from esp-idf)
            crate::rom::ets_delay_us(5);
            */
        }

        let conf = self.regs().clk_conf();

        conf.write(|w| unsafe {
            // UART clock source select. 1: 80Mhz, 2: 8Mhz, 3: XTAL.
            w.sclk_sel().bits(match config.clock_source {
                ClockSource::Apb => 1,
                ClockSource::RcFast => 2,
                //ClockSource::Xtal => 3,
            });
            // The numerator of the frequency divider factor.
            w.sclk_div_a().bits(0);
            // The denominator of the frequency divider factor
            w.sclk_div_b().bits(0);
            // The integral part of the frequency divider factor
            w.sclk_div_num().bits(clk_div as u8 - 1)
        });

        let divider = (clk << 4) / (config.baudrate * clk_div);

        let divider_integer = divider >> 4;
        let divider_frag = (divider & 0xf) as u8;

        self.regs().clkdiv().write(|w| unsafe {
            w.clkdiv()
                .bits(divider_integer as _)
                .frag()
                .bits(divider_frag)
        });

        self.sync_regs();

        #[cfg(feature = "unstable")]
        self.verify_baudrate(clk, config)?;

        Ok(())
    }

    // 2884``
    fn change_data_bits(&self, data_bits: DataBits) {
        self.regs()
            .conf0()
            // This register is used to set the length of data.
            .modify(|_, w| unsafe { w.bit_num().bits(data_bits as u8) });
    }

    // 2890
    fn change_parity(&self, parity: Parity) {
        // This register is used to configure the parity check mode
        self.regs().conf0().modify(|_, w| match parity {
            Parity::None => w.parity_en().clear_bit(),
            Parity::Even => w.parity_en().set_bit().parity().clear_bit(),
            Parity::Odd => w.parity_en().set_bit().parity().set_bit(),
        });
    }

    // 2898
    fn change_stop_bits(&self, stop_bits: StopBits) {
        // This register is used to set the length of stop bit.
        self.regs()
            .conf0()
            .modify(|_, w| unsafe { w.stop_bit_num().bits(stop_bits as u8 + 1) });
    }

    // 3000
    fn rxfifo_reset(&self) {
        fn rxfifo_rst(reg_block: &RegisterBlock, enable: bool) {
            reg_block.conf0().modify(|_, w| w.rxfifo_rst().bit(enable));
            sync_regs(reg_block);
        }

        rxfifo_rst(self.regs(), true);
        rxfifo_rst(self.regs(), false);
    }

    // 3010
    fn txfifo_reset(&self) {
        fn txfifo_rst(reg_block: &RegisterBlock, enable: bool) {
            reg_block.conf0().modify(|_, w| w.txfifo_rst().bit(enable));
            sync_regs(reg_block);
        }

        txfifo_rst(self.regs(), true);
        txfifo_rst(self.regs(), false);
    }

    fn verify_baudrate(&self, clk: u32, _config: &Config) -> Result<(), ConfigError> {
        // taken from https://github.com/espressif/esp-idf/blob/c5865270b50529cd32353f588d8a917d89f3dba4/components/hal/esp32c6/include/hal/uart_ll.h#L433-L444
        // (it's different for different chips)
        let clkdiv_reg = self.regs().clkdiv().read();
        let clkdiv_frag = clkdiv_reg.frag().bits() as u32;
        let clkdiv = clkdiv_reg.clkdiv().bits();

        cfg_if::cfg_if! {
            if #[cfg(any(esp32, esp32s2))] {
                let actual_baud = (clk << 4) / ((clkdiv << 4) | clkdiv_frag);
            } else if #[cfg(any(esp32c2, esp32c3, esp32s3))] {
                let sclk_div_num = self.regs().clk_conf().read().sclk_div_num().bits() as u32;
                let _actual_baud = (clk << 4) / ((((clkdiv as u32) << 4) | clkdiv_frag) * (sclk_div_num + 1));
            } else { // esp32c6, esp32h2
                let pcr = crate::peripherals::PCR::regs();
                let conf = if self.is_instance(unsafe { crate::peripherals::UART0::steal() }) {
                    pcr.uart(0).clk_conf()
                } else {
                    pcr.uart(1).clk_conf()
                };
                let sclk_div_num = conf.read().sclk_div_num().bits() as u32;
                let actual_baud = (clk << 4) / ((((clkdiv as u32) << 4) | clkdiv_frag) * (sclk_div_num + 1));
            }
        };

        /*
        match config.baudrate_tolerance {
            BaudrateTolerance::Exact => {
                let deviation = ((config.baudrate as i32 - actual_baud as i32).unsigned_abs()
                    * 100)
                    / actual_baud;
                // We tolerate deviation of 1% from the desired baud value, as it never will be
                // exactly the same
                if deviation > 1_u32 {
                    return Err(ConfigError::BaudrateNotAchievable);
                }
            }
            BaudrateTolerance::ErrorPercent(percent) => {
                let deviation = ((config.baudrate as i32 - actual_baud as i32).unsigned_abs()
                    * 100)
                    / actual_baud;
                if deviation > percent as u32 {
                    return Err(ConfigError::BaudrateNotAchievable);
                }
            }
            _ => {}
        }
        */

        Ok(())
    }

    // 3071
    fn current_symbol_length(&self) -> u8 {
        let conf0 = self.regs().conf0().read();
        // read length of data
        let data_bits = conf0.bit_num().bits() + 5; // 5 data bits are encoded as variant 0

        // read whether there is parity check
        let parity = conf0.parity_en().bit() as u8;
        // read length of stop bit
        let mut stop_bits = conf0.stop_bit_num().bits();

        match stop_bits {
            1 => {
                // workaround for hardware issue, when UART stop bit set as 2-bit mode.
                #[cfg(esp32)]
                if self.regs().rs485_conf().read().dl1_en().bit_is_set() {
                    stop_bits = 2;
                }
            }
            // esp-idf also counts 2 bits for settings 1.5 and 2 stop bits
            _ => stop_bits = 2,
        }

        1 + data_bits + parity + stop_bits
    }

    // 3121
    #[allow(clippy::useless_conversion)]
    fn tx_fifo_count(&self) -> u16 {
        // fn Stores the byte number of data in Tx-FIFO.
        u16::from(self.regs().status().read().txfifo_cnt().bits())
    }

    // 3125
    fn write_byte(&self, byte: u8) {
        self.regs()
            .fifo()
            .write(|w| unsafe { w.rxfifo_rd_byte().bits(byte) });
    }

    // 3131
    fn check_for_errors(&self) -> Result<(), RxError> {
        let errors = RxEvent::FifoOvf
            | RxEvent::FifoTout
            | RxEvent::GlitchDetected
            | RxEvent::FrameError
            | RxEvent::ParityError;
        let events = self.rx_events().intersection(errors);
        let result = rx_event_check_for_error(events);
        if result.is_err() {
            self.clear_rx_events(errors);
            if events.contains(RxEvent::FifoOvf) {
                self.rxfifo_reset();
            }
        }
        result
    }

    // 3150
    #[allow(clippy::unnecessary_cast)]
    fn rx_fifo_count(&self) -> u16 {
        self.regs().status().read().rxfifo_cnt().bits() as u16
    }

    // 3175
    fn write(&self, data: &[u8]) -> Result<usize, TxError> {
        if data.is_empty() {
            return Ok(0);
        }

        while self.tx_fifo_count() >= Info::UART_FIFO_SIZE {}

        let space = (Info::UART_FIFO_SIZE - self.tx_fifo_count()) as usize;
        let to_write = space.min(data.len());
        for &byte in &data[..to_write] {
            self.write_byte(byte);
        }

        Ok(to_write)
    }

    // 3204
    fn read_buffered(&self, buf: &mut [u8]) -> Result<usize, RxError> {
        // Get the count first, to avoid accidentally reading a corrupted byte received
        // after the error check.
        let to_read = (self.rx_fifo_count() as usize).min(buf.len());
        self.check_for_errors()?;

        for byte_into in buf[..to_read].iter_mut() {
            *byte_into = self.regs().fifo().read().rxfifo_rd_byte().bits();
        }

        Ok(to_read)
    }
}

// 3218
impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.register_block, other.register_block)
    }
}

// 3226
macro_rules! impl_instance {
    ($inst:ident, $peri:ident, $txd:ident, $rxd:ident) => {
        impl Instance for crate::peripherals::$inst<'_> {
            fn parts(&self) -> (&'static Info, &'static State) {
                #[crate::handler]
                pub(super) fn irq_handler() {
                    intr_handler(unsafe { &PERIPHERAL }, &STATE);
                }

                static STATE: State = State {
                    tx_waker: AtomicWaker::new(),
                    rx_waker: AtomicWaker::new(),
                    //is_rx_async: AtomicBool::new(false),
                    //is_tx_async: AtomicBool::new(false),
                };

                //static PERIPHERAL: Info = Info {
                static mut PERIPHERAL: Info = Info {
                    register_block: crate::peripherals::$inst::ptr(),
                    peripheral: crate::system::Peripheral::$peri,
                    async_handler: irq_handler,
                    interrupt: Interrupt::$inst,
                    tx_signal: OutputSignal::$txd,
                    rx_signal: InputSignal::$rxd,
                };
                unsafe { (&PERIPHERAL, &STATE) }
            }
        }
    };
}

// 3258
//impl_instance!(UART0, Uart0, U0TXD, U0RXD);
impl_instance!(UART1, Uart1, U1TXD, U1RXD);

// 3263
crate::any_peripheral! {
    /// Any UART peripheral.
    pub peripheral AnyUart<'d> {
        /*
        #[cfg(uart0)]
        Uart0(crate::peripherals::UART0<'d>),
        */
        //#[cfg(uart1)]
        #[cfg(soc_has_uart1)]
        Uart1(crate::peripherals::UART1<'d>),
    }
}

// 3275
impl Instance for AnyUart<'_> {
    #[inline]
    // 3277
    fn parts(&self) -> (&'static Info, &'static State) {
        /*
        match &self.0 {
            /*
            #[cfg(uart0)]
            AnyUartInner::Uart0(uart) => uart.parts(),
            */
            //#[cfg(uart1)]
            #[cfg(soc_has_uart1)]
            AnyUartInner::Uart1(uart) => uart.parts(),
        }
        */
        any::delegate!(self, uart => { uart.parts() })
    }
}
