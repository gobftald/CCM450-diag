//! # General Purpose Input/Output (GPIO)
//!
//! ## Overview
//!
//! Each pin can be used as a general-purpose I/O, or be connected to one or
//! more internal peripheral signals.

// 55
pub mod interconnect;
use interconnect::PeripheralOutput;

// 69
//pub(crate) mod interrupt;

// 74
//use interrupt::*; // we define GPIO_LOCK here
pub(crate) static GPIO_LOCK: RawMutex = RawMutex::new();

use esp_sync::RawMutex;

// 80
use crate::peripherals::{GPIO /*, handle_gpio_input, handle_gpio_output*/};
//pub use crate::soc::gpio::*; // everything is defined on metadata-generated
define_io_mux_signals!();

/// Represents a pin-peripheral connection that, when dropped, disconnects the
/// peripheral from the pin.
///
/// This only needs to be applied to output signals, as it's not possible to
/// connect multiple inputs to the same peripheral signal.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 95
pub(crate) struct PinGuard {
    pin: u8,
    //signal: OutputSignal,
}

// 102
impl PinGuard {
    // 103
    pub(crate) fn new(pin: AnyPin<'_> /* , signal: OutputSignal*/) -> Self {
        Self {
            pin: pin.number(),
            //signal,
        }
    }

    // 110
    pub(crate) fn new_unconnected(/*signal: OutputSignal*/) -> Self {
        Self {
            pin: u8::MAX,
            //signal,
        }
    }
}

// 127
impl Drop for PinGuard {
    fn drop(&mut self) {
        if self.pin != u8::MAX {
            let pin = unsafe { AnyPin::steal(self.pin) };
            //self.signal.disconnect_from(&pin);
            pin.disconnect_from_peripheral_output();
        }
    }
}

/// Digital input or output level.
///
/// `Level` can be used to control a GPIO output, and it can act as a peripheral
/// signal and be connected to peripheral inputs and outputs.
///
/// When connected to a peripheral
/// input, the peripheral will read the corresponding level from that signal.
///
/// When connected to a peripheral output, the level will be ignored.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 183
pub enum Level {
    /// Low
    Low,
    /// High
    High,
}

// 212
impl From<Level> for bool {
    fn from(level: Level) -> bool {
        match level {
            Level::Low => false,
            Level::High => true,
        }
    }
}

/// Pull setting for a GPIO.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 250
pub enum Pull {
    /// No pull
    None,
    /// Pull up
    Up,
    /// Pull down
    Down,
}

/// Drive strength (values are approximates)
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 262
pub enum DriveStrength {
    /// Drive strength of approximately 5mA.
    _5mA = 0,
    /// Drive strength of approximately 10mA.
    _10mA = 1,
    /// Drive strength of approximately 20mA.
    _20mA = 2,
    /// Drive strength of approximately 40mA.
    _40mA = 3,
}

/// Alternate functions
///
/// GPIO pins can be configured for various functions, such as GPIO
/// or being directly connected to a peripheral's signal like UART, SPI, etc.
/// The `AlternateFunction` enum allows selecting one of several functions that
/// a pin can perform, rather than using it as a general-purpose input or
/// output.
///
/// The different variants correspond to different functionality depending on
/// the chip and the specific pin. For more information, refer to your chip's
#[doc(hidden)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 287
pub enum AlternateFunction {
    /// Alternate function 0.
    _0 = 0,
    /// Alternate function 1.
    _1 = 1,
    /// Alternate function 2.
    _2 = 2,
    /// Alternate function 3.
    _3 = 3,
    /// Alternate function 4.
    _4 = 4,
    /// Alternate function 5.
    _5 = 5,
}

impl AlternateFunction {
    const GPIO: Self = match Self::const_try_from(property!("gpio.gpio_function")) {
        Ok(func) => func,
        Err(_) => ::core::panic!("Invalid GPIO function"),
    };

    const fn const_try_from(value: usize) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::_0),
            1 => Ok(Self::_1),
            2 => Ok(Self::_2),
            3 => Ok(Self::_3),
            4 => Ok(Self::_4),
            5 => Ok(Self::_5),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for AlternateFunction {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::const_try_from(value)
    }
}

/// Common trait implemented by pins
//pub trait Pin: Sealed {
// 368
pub trait Pin {
    /// GPIO number
    // 370
    fn number(&self) -> u8;

    /// Type-erase this pin into an [`AnyPin`].
    ///
    /// This function converts pin singletons (`GPIO0<'_>`, …), which are all
    /// different types, into the same type. It is useful for creating
    /// arrays of pins, or avoiding generics.
    ///
    /// ## Example
    ///
    /// ```rust, no_run
    /// use esp_hal::gpio::{AnyPin, Pin, Output, OutputConfig, Level};
    /// use esp_hal::delay::Delay;
    ///
    /// fn toggle_pins(pins: [AnyPin; 2], delay: &mut Delay) {
    ///     let [red, blue] = pins;
    ///     let mut red = Output::new(
    ///         red,
    ///         Level::High,
    ///         OutputConfig::default(),
    ///     );
    ///     let mut blue = Output::new(
    ///         blue,
    ///         Level::Low,
    ///         OutputConfig::default(),
    ///     );
    ///
    ///     loop {
    ///         red.toggle();
    ///         blue.toggle();
    ///         delay.delay_millis(500);
    ///     }
    /// }
    ///
    /// let pins: [AnyPin; 2] = [
    ///    peripherals.GPIO5.degrade(),
    ///    peripherals.GPIO6.degrade(),
    /// ];
    ///
    /// let mut delay = Delay::new();
    /// toggle_pins(pins, &mut delay);
    /// # Ok(())
    /// # }
    /// ```
    // 415
    fn degrade<'d>(self) -> AnyPin<'d>
    where
        Self: Sized + 'd,
    {
        unsafe { AnyPin::steal(self.number()) }
    }

    //fn output_signals(&self, _: private::Internal) -> &'static [(AlternateFunction, OutputSignal)];
    // 423
    fn output_signals(&self) -> &'static [(AlternateFunction, OutputSignal)];

    //fn input_signals(&self, _: private::Internal) -> &'static [(AlternateFunction, InputSignal)];
    // 426
    fn input_signals(&self) -> &'static [(AlternateFunction, InputSignal)];
}

/// Trait implemented by pins which can be used as inputs.
// 430
pub trait InputPin: Pin {}

/// Trait implemented by pins which can be used as outputs.
// 433
pub trait OutputPin: Pin {}

//#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, EnumCount)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 467
pub enum GpioBank {
    _0,
    /*
    #[cfg(gpio_bank_1)]
    _1,
    */
}

// 473
impl GpioBank {
    // 488
    fn write_out_en(self, word: u32, enable: bool) {
        if enable {
            self.write_out_en_set(word);
        } else {
            self.write_out_en_clear(word);
        }
    }

    // 496
    fn write_out_en_clear(self, word: u32) {
        match self {
            Self::_0 => GPIO::regs()
                // disable the output from the GPIO pin
                .enable_w1tc()
                .write(|w| unsafe { w.bits(word) }),
            /*
            #[cfg(gpio_bank_1)]
            Self::_1 => GPIO::regs()
                .enable1_w1tc()
                .write(|w| unsafe { w.bits(word) }),
            */
        };
    }

    // 508
    fn write_out_en_set(self, word: u32) {
        match self {
            Self::_0 => GPIO::regs()
                // enable the output from the GPIO pin
                .enable_w1ts()
                .write(|w| unsafe { w.bits(word) }),
            /*
            #[cfg(gpio_bank_1)]
            Self::_1 => GPIO::regs()
                .enable1_w1ts()
                .write(|w| unsafe { w.bits(word) }),
            */
        };
    }

    // 556
    fn write_output(self, word: u32, set: bool) {
        if set {
            self.write_output_set(word);
        } else {
            self.write_output_clear(word);
        }
    }

    // 564
    fn write_output_set(self, word: u32) {
        match self {
            // GPIO output set register for GPIO0-25
            Self::_0 => GPIO::regs().out_w1ts().write(|w| unsafe { w.bits(word) }),
            /*
            #[cfg(gpio_bank_1)]
            Self::_1 => GPIO::regs().out1_w1ts().write(|w| unsafe { w.bits(word) }),
            */
        };
    }

    // 572
    fn write_output_clear(self, word: u32) {
        match self {
            // GPIO output clear register for GPIO0-25
            Self::_0 => GPIO::regs().out_w1tc().write(|w| unsafe { w.bits(word) }),
            /*
            #[cfg(gpio_bank_1)]
            Self::_1 => GPIO::regs().out1_w1tc().write(|w| unsafe { w.bits(word) }),
            */
        };
    }
}

/// Any GPIO pin.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 584
pub struct AnyPin<'lt> {
    pub(crate) pin: u8,
    pub(crate) _lifetime: core::marker::PhantomData<&'lt mut ()>,
}

#[macro_export]
// 699
macro_rules! if_output_pin {
    // Base case: not an Output pin, substitute the else branch
    ({ $($then:tt)* } else { $($else:tt)* }) => { $($else)* };

    // First is an Output pin, skip checking and substitute the then branch
    (Output $(, $other:ident)* { $($then:tt)* } else { $($else:tt)* }) => { $($then)* };

    // First is not an Output pin, check the rest
    ($not:ident $(, $other:ident)* { $($then:tt)* } else { $($else:tt)* }) => {
        $crate::if_output_pin!($($other),* { $($then)* } else { $($else)* })
    };
}

#[macro_export]
// 730
macro_rules! io_type {
    (Input, $gpionum:literal) => {
        impl $crate::gpio::InputPin for paste::paste!( [<GPIO $gpionum>]<'_> ) {}
    };
    (Output, $gpionum:literal) => {
        impl $crate::gpio::OutputPin for paste::paste!( [<GPIO $gpionum>]<'_> ) {}
    };
}

/*
#[doc(hidden)]
#[macro_export]
// 767
macro_rules! gpio {
    (
        $(
            ($gpionum:literal, [$($type:tt),*]
                $(
                    ( $( $af_input_num:literal => $af_input_signal:ident )* )
                    ( $( $af_output_num:literal => $af_output_signal:ident )* )
                )?
            )
        )+
    ) => {
        paste::paste! {
            $(
                // 780
                impl<'d> [< GPIO $gpionum >]<'d> {
                    /// Split the pin into an input and output signal.
                    ///
                    /// Peripheral signals allow connecting peripherals together without using
                    /// external hardware.
                    ///
                    /// # Safety
                    ///
                    /// The caller must ensure that peripheral drivers don't configure the same
                    /// GPIO at the same time in multiple places. This includes clones of the
                    /// `InputSignal` struct, as well as the `OutputSignal` struct.
                    ///
                    /// ```rust, no_run
                    /// let (rx, tx) = unsafe { peripherals.GPIO2.split() };
                    /// // rx and tx can then be passed to different peripherals to connect them.
                    /// # Ok(())
                    /// # }
                    /// ```
                    // 800
                    pub unsafe fn split(self) -> ($crate::gpio::interconnect::InputSignal<'d>, $crate::gpio::interconnect::OutputSignal<'d>) {
                        use $crate::gpio::Pin;

                        // FIXME: we should implement this in the gpio macro for output pins, but we
                        // should also have an input-only alternative for pins that can't be used as
                        // outputs.

                        // This goes through AnyPin which calls `init_gpio` as needed.
                        unsafe { self.degrade().split() }
                    }
                }

                $(
                    $crate::io_type!($type, $gpionum);
                )*

                // 816
                impl $crate::gpio::Pin for [<GPIO $gpionum>]<'_> {
                    #[inline(always)]
                    // 817
                    fn number(&self) -> u8 {
                        $gpionum
                    }

                    //fn output_signals(&self, _: $crate::private::Internal) -> &'static [($crate::gpio::AlternateFunction, $crate::gpio::OutputSignal)] {
                    // 822
                    fn output_signals(&self) -> &'static [($crate::gpio::AlternateFunction, $crate::gpio::OutputSignal)] {
                        &[
                            $(
                                $(
                                    (
                                        $crate::gpio::AlternateFunction::[< _ $af_output_num >],
                                        $crate::gpio::OutputSignal::$af_output_signal
                                    ),
                                )*
                            )?
                        ]
                    }

                    //fn input_signals(&self, _: $crate::private::Internal) -> &'static [($crate::gpio::AlternateFunction, $crate::gpio::InputSignal)] {
                    // 835
                    fn input_signals(&self) -> &'static [($crate::gpio::AlternateFunction, $crate::gpio::InputSignal)] {
                        &[
                            $(
                                $(
                                    (
                                        $crate::gpio::AlternateFunction::[< _ $af_input_num >],
                                        $crate::gpio::InputSignal::$af_input_signal
                                    ),
                                )*
                            )?
                        ]
                    }
                }

                // 849
                impl<'lt> From<[<GPIO $gpionum>]<'lt>> for $crate::gpio::AnyPin<'lt> {
                    fn from(pin: [<GPIO $gpionum>]<'lt>) -> Self {
                        $crate::gpio::Pin::degrade(pin)
                    }
                }
            )+

            // 856
            impl $crate::gpio::AnyPin<'_> {
                /// Conjure a new GPIO pin out of thin air.
                ///
                /// # Safety
                ///
                /// The caller must ensure that only one instance of a pin is in use at one time.
                ///
                /// # Panics
                ///
                /// Panics if the pin with the given number does not exist.
                // 866
                pub unsafe fn steal(pin: u8) ->  Self {
                    const PINS: &[u8] = &[$($gpionum),*];
                    assert!(PINS.contains(&pin), "Pin {} does not exist", pin);
                    Self { pin, _lifetime: core::marker::PhantomData }
                }

                /// Unsafely clone the pin.
                ///
                /// # Safety
                ///
                /// Ensure that only one instance of a pin is in use at one time.
                // 877
                pub unsafe fn clone_unchecked(&self) -> Self {
                    Self {
                        pin: self.pin,
                        _lifetime: core::marker::PhantomData,
                    }
                }

                // 890
                pub(crate) fn is_output(&self) -> bool {
                    match self.pin {
                        $(
                            $gpionum => $crate::if_output_pin!($($type),* { true } else { false }),
                        )+
                        _ => false,
                    }
                }
            }

            // These macros call the code block on the actually contained GPIO pin.

            // 903
            macro_rules! handle_gpio_output {
                ($this:expr, $inner:ident, $code:tt) => {
                    match $this.number() {
                        $(
                            $gpionum => $crate::if_output_pin!($($type),* {{
                                #[allow(unused_mut)]
                                let mut $inner = unsafe { $crate::peripherals::[<GPIO $gpionum>]::steal() };
                                $code
                            }} else {{
                                panic!("Unsupported")
                            }}),
                        )+
                        _ => unreachable!(),
                    }
                }
            }

            // 921
            macro_rules! handle_gpio_input {
                ($this:expr, $inner:ident, $code:tt) => {
                    match $this.number() {
                        $(
                            $gpionum => {{
                                #[allow(unused_mut)]
                                let mut $inner = unsafe { $crate::peripherals::[<GPIO $gpionum>]::steal() };
                                $code
                            }},
                        )+
                        _ => unreachable!(),
                    }
                }
            }

            // 936
            pub(crate) use handle_gpio_output;
            pub(crate) use handle_gpio_input;
        }
    };
}
*/

/// The drive mode of the output pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 993

pub enum DriveMode {
    /// Push-pull output.
    ///
    /// The driver actively sets the output voltage level for both high and low
    /// logical [`Level`]s.
    PushPull,

    /// Open drain output.
    ///
    /// The driver actively pulls the output voltage level low for the low
    /// logical [`Level`], but leaves the high level floating, which is then
    /// determined by external hardware, or internal pull-up/pull-down
    /// resistors.
    #[cfg_attr(
        feature = "unstable",
        doc = "\n\nEnable the input related functionality by using [Output::into_flex] and enabling input via [Flex::set_input_enable]"
    )]
    OpenDrain,
}

/// Output pin configuration.
///
/// This struct is used to configure the drive mode, drive strength, and pull
/// direction of an output pin. By default, the configuration is set to:
/// - Drive mode: [`DriveMode::PushPull`]
/// - Drive strength: [`DriveStrength::_20mA`]
/// - Pull direction: [`Pull::None`] (no pull resistors connected)
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
//#[derive(Debug, Clone, Copy, PartialEq, Eq, procmacros::BuilderLite)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
// 1023
pub struct OutputConfig {
    /// Output drive mode.
    drive_mode: DriveMode,

    /// Pin drive strength.
    drive_strength: DriveStrength,

    /// Pin pull direction.
    pull: Pull,
}

// 1034
impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            drive_mode: DriveMode::PushPull,
            drive_strength: DriveStrength::_20mA,
            pull: Pull::Up,
        }
    }
}

/// Push-pull digital output.
///
/// This driver configures the GPIO pin to be an output driver.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1049
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    /// Creates a new GPIO output driver.
    ///
    /// The `initial_level` parameter sets the initial output level of the pin.
    /// The `config` parameter sets the drive mode, drive strength, and pull
    /// direction of the pin.
    ///
    /// ## Example
    ///
    /// The following example configures `GPIO5` to pulse a LED once. The
    /// example assumes that the LED is connected such that it is on when
    /// the pin is low.
    ///
    /// ```rust, no_run
    /// use esp_hal::gpio::{Level, Output, OutputConfig};
    /// use esp_hal::delay::Delay;
    ///
    /// fn blink_once(led: &mut Output<'_>, delay: &mut Delay) {
    ///     led.set_low();
    ///     delay.delay_millis(500);
    ///     led.set_high();
    /// }
    ///
    /// let config = OutputConfig::default();
    /// let mut led = Output::new(peripherals.GPIO5, Level::High, config);
    /// let mut delay = Delay::new();
    ///
    /// blink_once(&mut led, &mut delay);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    // 1089
    pub fn new(pin: impl OutputPin + 'd, initial_level: Level, config: OutputConfig) -> Self {
        // Set up the pin
        let mut this = Self {
            pin: Flex::new(pin),
        };
        this.set_level(initial_level);
        this.apply_config(&config);
        this.pin.pin.set_output_enable(true);

        this
    }

    /// Change the configuration.
    #[inline]
    // 1127
    pub fn apply_config(&mut self, config: &OutputConfig) {
        self.pin.apply_output_config(config)
    }

    /// Set the output level.
    #[inline]
    // 1145
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_level(level)
    }
}

/// Input pin configuration.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
//#[derive(Debug, Clone, Copy, PartialEq, Eq, procmacros::BuilderLite)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
// 1197
pub struct InputConfig {
    /// Initial pull of the pin.
    pull: Pull,
}

// instead of procmacros::BuilderLite
impl InputConfig {
    pub fn with_pull(&mut self, pull: Pull) -> Self {
        self.pull = pull;
        *self
    }
}

// 1202
impl Default for InputConfig {
    fn default() -> Self {
        Self { pull: Pull::None }
    }
}

/// Flexible pin driver.
///
/// This pin driver can act as either input, or output, or both at the same
/// time. The input and output are (not counting the shared pull direction)
/// separately configurable, and they have independent enable states.
///
/// Enabling the input stage does not change the output stage, and vice versa.
/// Disabling the input or output stages don't forget their configuration.
/// Disabling the output stage will not change the output level, but it will
/// disable the driver.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1447
pub struct Flex<'d> {
    pin: AnyPin<'d>,
}

// 1453
impl<'d> Flex<'d> {
    /// Create flexible pin driver for a [Pin].
    /// No mode change happens.
    #[inline]
    // 1458
    pub fn new(pin: impl Pin + 'd) -> Self {
        let pin = pin.degrade();

        // Before each use, reset the GPIO to a known state.
        pin.init_gpio();

        Self { pin }
    }

    // Output functions

    /// Applies the given output configuration to the pin.
    ///
    /// This function does not set the pin to output (i.e. it does not enable
    /// the output driver). Note that the pull direction is common between
    /// the input and output configuration.
    // 1586
    pub fn apply_output_config(&mut self, config: &OutputConfig) {
        self.pin.apply_output_config(config);
    }

    /// Set the output level.
    #[inline]
    // 1620
    pub fn set_level(&mut self, level: Level) {
        self.pin.set_output_high(level.into());
    }
}

// 1778
impl<'lt> AnyPin<'lt> {
    // 1779
    fn bank(&self) -> GpioBank {
        /*
        #[cfg(gpio_bank_1)]
        if self.number() >= 32 {
            return GpioBank::_1;
        }
        */

        GpioBank::_0
    }

    #[inline]
    /// Resets the GPIO to a known state.
    ///
    /// This function needs to be called before using the GPIO pin:
    /// - Before converting it into signals
    /// - Before using it as an input or output
    // 1794
    pub(crate) fn init_gpio(&self) {
        self.set_output_enable(false); // if ouptut -> disable
        // if input -> enable

        GPIO::regs()
            // set func_out_sel_cfg to GPIO -> periheral output signal will be connected to GPIO output
            .func_out_sel_cfg(self.number() as usize)
            //.modify(|_, w| unsafe { w.out_sel().bits(OutputSignal::GPIO as OutputSignalType) });
            .modify(|_, w| unsafe { w.out_sel().bits(OutputSignal::GPIO as _) });

        // Use RMW to not overwrite sleep configuration
        io_mux_reg(self.number()).modify(|_, w| unsafe {
            // bypass GPIO matrix for peripheral output signals
            //w.mcu_sel().bits(GPIO_FUNCTION as u8);
            w.mcu_sel().bits(AlternateFunction::GPIO as u8);
            // input disabled on this pin
            // this is the only way to select pure GPIO input
            w.fun_ie().clear_bit();
            // don't put pin into sleep mode
            w.slp_sel().clear_bit()
        });
    }

    /// Split the pin into an input and output signal.
    ///
    /// Peripheral signals allow connecting peripherals together without
    /// using external hardware.
    ///
    /// Creating an input signal enables the pin's input buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that peripheral drivers don't configure the same
    /// GPIO at the same time in multiple places. This includes clones of the
    /// `InputSignal` struct, as well as the `OutputSignal` struct.
    ///
    /// # Panics
    ///
    /// This function panics if the pin is not an output pin.
    ///
    /// ```rust, no_run
    /// # use esp_hal::gpio::{AnyPin, Pin};
    /// let pin1 = peripherals.GPIO1.degrade();
    /// let (input, output) = unsafe { pin1.split() };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    // 1839
    pub unsafe fn split(
        self,
    ) -> (
        interconnect::InputSignal<'lt>,
        interconnect::OutputSignal<'lt>,
    ) {
        assert!(self.is_output());

        // Before each use, reset the GPIO to a known state.
        self.init_gpio();
        self.set_input_enable(true);

        let (input, output) = unsafe { self.split_no_init() };

        // We don't know if the input signal(s) will support bypassing the GPIO matrix.
        // Since the bypass option is common between input and output halves of
        // a single GPIO, we can't assume anything about the output, either.
        let output = output.with_gpio_matrix_forced(true);

        (input, output)
    }

    // 1907
    unsafe fn split_no_init(
        self,
    ) -> (
        interconnect::InputSignal<'lt>,
        interconnect::OutputSignal<'lt>,
    ) {
        // A GPIO input signal can be connected to any number of peripheral inputs.
        let input = interconnect::InputSignal::new(unsafe { self.clone_unchecked() });
        let output = interconnect::OutputSignal::new(self);

        // Since InputSignal can be cloned, we have no way of knowing how many signals
        // end up being configured, and in what order. If multiple signals are
        // passed to peripherals, and one of them would allow GPIO alternate
        // function configurations, it would mean that the GPIO MCU_SEL bit's
        // final value would depend on the order of operations.
        let input = input.with_gpio_matrix_forced(true);

        (input, output)
    }

    #[inline]
    // 1927
    pub(crate) fn set_alternate_function(&self, alternate: AlternateFunction) {
        io_mux_reg(self.number()).modify(|_, w| unsafe { w.mcu_sel().bits(alternate as u8) });
    }

    /// Enable or disable the GPIO pin output buffer.
    #[inline]
    // 1939
    pub(crate) fn set_output_enable(&self, enable: bool) {
        assert!(self.is_output() || !enable);
        self.bank().write_out_en(self.mask(), enable);
    }

    /// Enable input for the pin
    #[inline]
    // 1946
    pub(crate) fn set_input_enable(&self, on: bool) {
        // input enabled on this pin
        // this is the only way to select pure GPIO input
        io_mux_reg(self.number()).modify(|_, w| w.fun_ie().bit(on));
    }

    #[inline]
    // 1951
    pub(crate) fn apply_input_config(&self, config: &InputConfig) {
        let pull_up = config.pull == Pull::Up;
        let pull_down = config.pull == Pull::Down;

        io_mux_reg(self.number()).modify(|_, w| {
            w.fun_wpd().bit(pull_down);
            w.fun_wpu().bit(pull_up)
        });
    }

    // 1968
    fn with_gpio_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // If the pin is listening, we need to take a critical section to prevent racing
        // with the interrupt handler.
        if is_int_enabled(self.number()) {
            GPIO_LOCK.lock(f)
        } else {
            f()
        }
    }

    #[inline]
    // 2014
    fn apply_output_config(&self, config: &OutputConfig) {
        let pull_up = config.pull == Pull::Up;
        let pull_down = config.pull == Pull::Down;

        io_mux_reg(self.number()).modify(|_, w| {
            // Select the drive strength of the pad. 0: ~5 mA; 1: ~10mA; 2: ~20mA; 3: ~40mA."]
            unsafe { w.fun_drv().bits(config.drive_strength as u8) };
            // Pull-up enable of the pad. 1: internal pull-up enabled; 0: internal pull-up disabled.
            w.fun_wpu().bit(pull_up);
            // Field `FUN_WPD` writer - Pull-down enable of the pad.
            // 1: internal pull-down enabled; 0: internal pull-down disabled
            w.fun_wpd().bit(pull_down);
            w
        });

        self.with_gpio_lock(|| {
            GPIO::regs().pin(self.number() as usize).modify(|_, w| {
                // set this bit to select pad driver. 1:open-drain. 0:normal
                w.pad_driver()
                    .bit(config.drive_mode == DriveMode::OpenDrain)
            });
        });
    }

    #[inline]
    // 2037
    fn mask(&self) -> u32 {
        1 << (self.number() % 32)
    }

    /// Set the pin's level to high or low
    #[inline]
    // 2049
    pub(crate) fn set_output_high(&self, high: bool) {
        self.bank().write_output(self.mask(), high);
    }
}

// 2060
impl Pin for AnyPin<'_> {
    #[inline(always)]
    fn number(&self) -> u8 {
        self.pin
    }

    // 2066
    //fn output_signals(&self, _: private::Internal) -> &'static [(AlternateFunction, OutputSignal)] {
    fn output_signals(&self) -> &'static [(AlternateFunction, OutputSignal)] {
        /*
        handle_gpio_output!(self, target, {
            //Pin::output_signals(&target, private::Internal)
            Pin::output_signals(&target)
        })
        */
        for_each_gpio! {
            (all $( ($n:literal, $gpio:ident $in_afs:tt $out_afs:tt ($input:tt [$($is_output:ident)?]) ) ),* ) => {
                match self.number() {
                    $($(
                        $n => {
                            crate::ignore!($is_output);
                            let inner = unsafe { crate::peripherals::$gpio::steal() };
                            return Pin::output_signals(&inner/*, private*/);
                        }
                    )?)*
                    other => panic!("Pin {} is not an OutputPin", other)
                }
            };
        }
    }

    // 2072
    //fn input_signals(&self, _: private::Internal) -> &'static [(AlternateFunction, InputSignal)] {
    fn input_signals(&self) -> &'static [(AlternateFunction, InputSignal)] {
        /*
        handle_gpio_input!(self, target, {
            //Pin::input_signals(&target, private::Internal)
            Pin::input_signals(&target)
        })
        */
        for_each_gpio! {
            (all $( ($n:literal, $gpio:ident $in_afs:tt $out_afs:tt ([$($is_input:ident)?] $output:tt) ) ),* ) => {
                match self.number() {
                    $($(
                        $n => {
                            crate::ignore!($is_input);
                            let inner = unsafe { crate::peripherals::$gpio::steal() };
                            return Pin::input_signals(&inner/*, private*/);
                        }
                    )?)*
                    other => panic!("Pin {} is not an InputPin", other)
                }
            };
        }
    }
}

// 2079
impl InputPin for AnyPin<'_> {}
impl OutputPin for AnyPin<'_> {}

impl AnyPin<'_> {
    pub unsafe fn steal(pin: u8) -> Self {
        for_each_gpio! {
            (all $( ($n:literal $($any:tt)*) ),*) => { const PINS: &[u8] = &[ $($n),* ]; };
        };
        assert!(PINS.contains(&pin), "Pin {} does not exist", pin);
        Self {
            pin,
            _lifetime: core::marker::PhantomData,
        }
    }

    pub unsafe fn clone_unchecked(&self) -> Self {
        Self {
            pin: self.pin,
            _lifetime: core::marker::PhantomData,
        }
    }

    pub(crate) fn is_output(&self) -> bool {
        for_each_gpio! {
            (all $( ($n:literal, $gpio:ident $in_afs:tt $out_afs:tt ($input:tt [$($is_output:ident)?]) ) ),* ) => {
                return match self.number() {
                    $($(
                        // This code is generated if the Output attribute is present
                        $n => {
                            crate::ignore!($is_output);
                            true
                        }
                    )?)*
                    _other => false,
                };
            };
        }
    }
}

// 2144
fn is_int_enabled(gpio_num: u8) -> bool {
    // Interrupt enable bits: bit13: CPU interrupt enabled
    GPIO::regs().pin(gpio_num as usize).read().int_ena().bits() != 0
}

for_each_gpio! {
    ($n:literal, $gpio:ident ($( $af_input_num:ident => $af_input_signal:ident )*) ($( $af_output_num:ident => $af_output_signal:ident )*) $attrs:tt) => {
        impl<'d> crate::peripherals::$gpio<'d> {
            #[allow(unused)]
            pub(crate) const NUMBER: u8 = $n;

            //#[procmacros::doc_replace]
            /// Split the pin into an input and output signal.
            ///
            /// Peripheral signals allow connecting peripherals together without using
            /// external hardware.
            ///
            /// # Safety
            ///
            /// The caller must ensure that peripheral drivers don't configure the same
            /// GPIO at the same time in multiple places. This includes clones of the
            /// `InputSignal` struct, as well as the `OutputSignal` struct.
            ///
            /// ```rust, no_run
            /// # {before_snippet}
            /// #
            /// let (rx, tx) = unsafe { peripherals.GPIO2.split() };
            /// // rx and tx can then be passed to different peripherals to connect them.
            /// #
            /// # {after_snippet}
            /// ```
            //#[instability::unstable]
            pub unsafe fn split(self) -> (interconnect::InputSignal<'d>, interconnect::OutputSignal<'d>) {
                // FIXME: we should implement this in the gpio macro for output pins, but we
                // should also have an input-only alternative for pins that can't be used as
                // outputs.

                // This goes through AnyPin which calls `init_gpio` as needed.
                unsafe { self.degrade().split() }
            }
        }

        impl Pin for crate::peripherals::$gpio<'_> {
            #[inline(always)]
            fn number(&self) -> u8 {
                $n
            }

            fn output_signals(&self/*, _: crate::private::Internal*/) -> &'static [(AlternateFunction, OutputSignal)] {
                &[$(
                        (AlternateFunction::$af_output_num, OutputSignal::$af_output_signal),
                )*]
            }

            fn input_signals(&self/*, _: crate::private::Internal*/) -> &'static [(AlternateFunction, InputSignal)] {
                &[$(
                        (AlternateFunction::$af_input_num, InputSignal::$af_input_signal),
                )*]
            }
        }

        impl<'lt> From<crate::peripherals::$gpio<'lt>> for AnyPin<'lt> {
            fn from(pin: crate::peripherals::$gpio<'lt>) -> Self {
                Pin::degrade(pin)
            }
        }
    };
}

define_io_mux_reg!();
