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

// 51
use crate::{
    clock::Clocks,
    gpio::{
        interconnect::{PeripheralInput, PeripheralOutput},
        InputConfig, InputSignal, OutputConfig, OutputSignal, PinGuard, Pull,
    },
    pac::uart0::RegisterBlock,
    system::PeripheralClockControl,
};

/// UART clock source
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 150
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
// 173
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
// 193
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
// 212
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
// 299
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

// 328
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

// 345
impl Config {
    // 346
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
}

/// UART Receive part configuration.
//#[derive(Debug, Clone, Copy, procmacros::BuilderLite)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 363
pub struct RxConfig {
    /// Threshold level at which the RX FIFO is considered full.
    fifo_full_threshold: u16,
    /// Optional timeout value for RX operations.
    timeout: Option<u8>,
}

// 370
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
// 385
pub struct TxConfig {
    /// Threshold level at which the TX FIFO is considered empty.
    fifo_empty_threshold: u16,
}

// 390
impl Default for TxConfig {
    fn default() -> TxConfig {
        TxConfig {
            // see <https://github.com/espressif/esp-idf/blob/8760e6d2a/components/esp_driver_uart/src/uart.c#L59>
            fifo_empty_threshold: 10,
        }
    }
}

// 432
//struct UartBuilder<'d, Dm: DriverMode> {
struct UartBuilder<'d> {
    uart: AnyUart<'d>,
    //phantom: PhantomData<Dm>,
}

// 437
/*
impl<'d, Dm> UartBuilder<'d, Dm>
where
    Dm: DriverMode,
{
*/
impl<'d> UartBuilder<'d> {
    // 441
    fn new(uart: impl Instance + 'd) -> Self {
        Self {
            uart: uart.degrade(),
            //phantom: PhantomData,
        }
    }

    // 448
    //fn init(self, config: Config) -> Result<Uart<'d, Dm>, ConfigError> {}
    fn init(self, config: Config) -> Result<Uart<'d>, ConfigError> {
        let tx_pin = PinGuard::new_unconnected(self.uart.info().tx_signal);

        let mut serial = Uart {
            rx: UartRx {
                uart: unsafe { self.uart.clone_unchecked() },
                //phantom: PhantomData,
                // Uart0 is KEEP_ENABLED
                //guard: rx_guard,
            },
            tx: UartTx {
                uart: self.uart,
                //phantom: PhantomData
                // Uart0 is KEEP_ENABLED,
                //guard: tx_guard,
                //rts_pin,
                tx_pin,
            },
        };
        serial.init(config)?;

        Ok(serial)
    }
}

// 490
//pub struct Uart<'d, Dm: DriverMode> {}
pub struct Uart<'d> {
    //rx: UartRx<'d, Dm>,
    rx: UartRx<'d>,
    //tx: UartTx<'d, Dm>,
    tx: UartTx<'d>,
}

/// UART (Transmit)
// 497
//pub struct UartTx<'d, Dm: DriverMode> {
pub struct UartTx<'d> {
    uart: AnyUart<'d>,
    //phantom: PhantomData<Dm>,
    // Uart0 is KEEP_ENABLED
    //guard: PeripheralGuard,
    //rts_pin: PinGuard,
    tx_pin: PinGuard,
}

/// UART (Receive)
// 507
//pub struct UartRx<'d, Dm: DriverMode> {
pub struct UartRx<'d> {
    uart: AnyUart<'d>,
    //phantom: PhantomData<Dm>,
    // Uart0 is KEEP_ENABLED
    //guard: PeripheralGuard,
}

/// A configuration error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 517
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

// 738
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
    // 755
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
    // 782
    pub fn apply_config(&mut self, config: &Config) -> Result<(), ConfigError> {
        self.uart
            .info()
            .set_tx_fifo_empty_threshold(config.tx.fifo_empty_threshold)?;
        self.uart.info().txfifo_reset();
        Ok(())
    }

    /// Disables all TX-related interrupts for this UART instance.
    ///
    /// This function clears and disables the `transmit FIFO empty` interrupt,
    /// `transmit break done`, `transmit break idle done`, and `transmit done`
    /// interrupts.
    // 863
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

    // 879
    fn regs(&self) -> &RegisterBlock {
        self.uart.info().regs()
    }
}

#[inline(always)]
// 885
fn sync_regs(_register_block: &RegisterBlock) {
    let update_reg = _register_block.id();

    // Software write 1 would synchronize registers into UART Core clock domain
    update_reg.modify(|_, w| w.reg_update().set_bit());

    // and would be cleared by hardware after synchronization is done.
    while update_reg.read().reg_update().bit_is_set() {
        // wait
    }
}

// 1085
/*
impl<'d, Dm> UartRx<'d, Dm>
where
    Dm: DriverMode,
*/
impl<'d> UartRx<'d> {
    // 1089
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
    // 1117
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
    // 1135
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
    // 1213
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

// 1230
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
    // 1251
    //pub fn new(uart: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
    pub fn new(uart: impl Instance + 'd, config: Config) -> Result<Self, ConfigError> {
        UartBuilder::new(uart).init(config)
    }
}

/*
impl<'d, Dm> Uart<'d, Dm>
where
    Dm: DriverMode,
*/
// 1477
impl<'d> Uart<'d> {
    /// Assign the RX pin for UART instance.
    ///
    /// Sets the specified pin to input and connects it to the UART RX signal.
    ///
    /// Note: when you listen for the output of the UART peripheral, you should
    /// configure the driver side (i.e. the TX pin), or ensure that the line is
    /// initially high, to avoid receiving a non-data byte caused by an
    /// initial low signal level.
    // 1481
    pub fn with_rx(mut self, rx: impl PeripheralInput<'d>) -> Self {
        self.rx = self.rx.with_rx(rx);
        self
    }

    /// Assign the TX pin for UART instance.
    ///
    /// Sets the specified pin to push-pull output and connects it to the UART
    /// TX signal.
    // 1498
    pub fn with_tx(mut self, tx: impl PeripheralOutput<'d>) -> Self {
        self.tx = self.tx.with_tx(tx);
        self
    }

    // 1515
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
    // 1599
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
    // 1634
    pub fn split(self) -> (UartRx<'d>, UartTx<'d>) {
        (self.rx, self.tx)
    }

    #[inline(always)]
    // 1703
    fn init(&mut self, config: Config) -> Result<(), ConfigError> {
        crate::peripherals::SYSTEM::regs()
            // enable the clock for UART RAM
            .perip_clk_en0()
            .modify(|_, w| w.uart_mem_clk_en().set_bit());

        self.uart_peripheral_reset();

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

    #[inline(always)]
    // 1753
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
}

// 2427
/// A peripheral singleton compatible with the UART driver.
//pub trait Instance: crate::private::Sealed + IntoAnyUart
pub trait Instance: IntoAnyUart {
    /// Returns the peripheral data and state describing this UART instance.
    // 2430
    fn parts(&self) -> (&'static Info, &'static State);

    /// Returns the peripheral data describing this UART instance.
    #[inline(always)]
    // 2435
    fn info(&self) -> &'static Info {
        self.parts().0
    }
}

/// Peripheral data describing a particular UART instance.
#[non_exhaustive]
// 2450
pub struct Info {
    /// Pointer to the register block for this UART instance.
    ///
    /// Use [Self::register_block] to access the register block.
    pub register_block: *const RegisterBlock,

    /// The system peripheral marker.
    pub peripheral: crate::system::Peripheral,

    /// TX pin
    pub tx_signal: OutputSignal,

    /// RX pin
    pub rx_signal: InputSignal,
}

/// Peripheral state for a UART instance.
#[non_exhaustive]
// 2481
pub struct State {}

// 2495
impl Info {
    // Currently we don't support merging adjacent FIFO memory, so the max size is
    // 128 bytes, the max threshold is 127 bytes.
    //const UART_FIFO_SIZE: u16 = 128;
    const RX_FIFO_MAX_THRHD: u16 = 127;
    const TX_FIFO_MAX_THRHD: u16 = Self::RX_FIFO_MAX_THRHD;

    /// Returns the register block for this UART instance.
    // 2503
    pub fn regs(&self) -> &RegisterBlock {
        unsafe { &*self.register_block }
    }

    // 2576
    fn apply_config(&self, config: &Config) -> Result<(), ConfigError> {
        config.validate()?;
        self.change_baud(config)?;
        self.change_data_bits(config.data_bits);
        self.change_parity(config.parity);
        self.change_stop_bits(config.stop_bits);
        //self.change_flow_control(config.sw_flow_ctrl, config.hw_flow_ctrl);

        Ok(())
    }

    /// Configures the RX-FIFO threshold
    ///
    /// ## Errors
    ///
    /// [ConfigError::UnsupportedRxFifoThreshold] if the provided value exceeds
    /// [`Info::RX_FIFO_MAX_THRHD`].
    // 2698
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

    /// Configures the TX-FIFO threshold
    ///
    /// ## Errors
    ///
    /// [ConfigError::UnsupportedTxFifoThreshold] if the provided value exceeds
    /// [`Info::TX_FIFO_MAX_THRHD`].
    // 2722
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
    // 2748
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
            reg_thrhd.modify(|_, w| unsafe { w.rx_tout_thrhd().bits(timeout_reg) });
        }

        let reg_en = register_block.conf1();
        // This is the enble bit for uart receiver's timeout function.
        reg_en.modify(|_, w| w.rx_tout_en().bit(timeout.is_some()));

        self.sync_regs();

        Ok(())
    }

    // 2801
    fn sync_regs(&self) {
        sync_regs(self.regs());
    }

    // 2805
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

    // 2886
    fn change_data_bits(&self, data_bits: DataBits) {
        self.regs()
            .conf0()
            // This register is used to set the length of data.
            .modify(|_, w| unsafe { w.bit_num().bits(data_bits as u8) });
    }

    // 2892
    fn change_parity(&self, parity: Parity) {
        // This register is used to configure the parity check mode
        self.regs().conf0().modify(|_, w| match parity {
            Parity::None => w.parity_en().clear_bit(),
            Parity::Even => w.parity_en().set_bit().parity().clear_bit(),
            Parity::Odd => w.parity_en().set_bit().parity().set_bit(),
        });
    }

    // 2900
    fn change_stop_bits(&self, stop_bits: StopBits) {
        // This register is used to set the length of stop bit.
        self.regs()
            .conf0()
            .modify(|_, w| unsafe { w.stop_bit_num().bits(stop_bits as u8 + 1) });
    }

    // 3002
    fn rxfifo_reset(&self) {
        fn rxfifo_rst(reg_block: &RegisterBlock, enable: bool) {
            reg_block.conf0().modify(|_, w| w.rxfifo_rst().bit(enable));
            sync_regs(reg_block);
        }

        rxfifo_rst(self.regs(), true);
        rxfifo_rst(self.regs(), false);
    }

    // 3013
    fn txfifo_reset(&self) {
        fn txfifo_rst(reg_block: &RegisterBlock, enable: bool) {
            reg_block.conf0().modify(|_, w| w.txfifo_rst().bit(enable));
            sync_regs(reg_block);
        }

        txfifo_rst(self.regs(), true);
        txfifo_rst(self.regs(), false);
    }

    // 3073
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
}

// 3228
macro_rules! impl_instance {
    ($inst:ident, $peri:ident, $txd:ident, $rxd:ident) => {
        impl Instance for crate::peripherals::$inst<'_> {
            fn parts(&self) -> (&'static Info, &'static State) {
                static STATE: State = State {};

                //static PERIPHERAL: Info = Info {
                static mut PERIPHERAL: Info = Info {
                    register_block: crate::peripherals::$inst::ptr(),
                    peripheral: crate::system::Peripheral::$peri,
                    tx_signal: OutputSignal::$txd,
                    rx_signal: InputSignal::$rxd,
                };
                unsafe { (&PERIPHERAL, &STATE) }
            }
        }
    };
}

// 3260
impl_instance!(UART0, Uart0, U0TXD, U0RXD);

// 3265
crate::any_peripheral! {
    /// Any UART peripheral.
    pub peripheral AnyUart<'d> {
        #[cfg(uart0)]
        Uart0(crate::peripherals::UART0<'d>),
    }
}

// 3277
impl Instance for AnyUart<'_> {
    #[inline]
    // 3279
    fn parts(&self) -> (&'static Info, &'static State) {
        match &self.0 {
            #[cfg(uart0)]
            AnyUartInner::Uart0(uart) => uart.parts(),
        }
    }
}
