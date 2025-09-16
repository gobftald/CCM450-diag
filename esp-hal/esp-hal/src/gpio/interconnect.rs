//! # Peripheral signal interconnect using the GPIO matrix.
//!
//! The GPIO matrix offers flexible connection options between GPIO pins and
//! peripherals. This module offers capabilities not covered by GPIO pin types
//! and drivers, like routing fixed logic levels to peripheral inputs, or
//! inverting input and output signals.
//!
//! > Note that routing a signal through the GPIO matrix adds some latency to
//! > the signal. This is not a problem for most peripherals, but it can be an
//! > issue for high-speed peripherals like SPI or I2S. `esp-hal` tries to
//! > bypass the GPIO matrix when possible (e.g. when the pin can be configured
//! > as a suitable Alternate Function for the peripheral signal, and other
//! > settings are compatible), but silently falls back to the GPIO matrix for
//! > flexibility.
//!
//! The GPIO drivers implement IO MUX and pin functionality (input/output
//! buffers, pull resistors, etc.). The GPIO matrix is represented by signals
//! and the [`PeripheralInput`] and [`PeripheralOutput`] traits. There is some
//! overlap between them: signal routing depends on what type is passed to a
//! peripheral driver's pin setter functions.
//!
//! ## Signals
//!
//! GPIO signals are represented by the [`InputSignal`] and [`OutputSignal`]
//! structs. Peripheral drivers accept [`PeripheralInput`] and
//! [`PeripheralOutput`] implementations which are implemented for anything that
//! can be converted into the signal types:
//! - GPIO pins and drivers
//! - A fixed logic [`Level`]
//! - [`NoPin`]
//!
//! Note that some of these exist for convenience only. `Level` is meaningful as
//! a peripheral input, but not as a peripheral output. `NoPin` is a placeholder
//! for when a peripheral driver does not require a pin, but the API requires
//! one. It is equivalent to [`Level::Low`].
//!
//! ....
//!
//! ## Connection rules
//!
//! Peripheral signals and GPIOs can be connected with the following
//! constraints:
//!
//! - A peripheral input signal must be driven by exactly one signal, which can
//!   be a GPIO input or a constant level.
//! - A peripheral output signal can be connected to any number of GPIOs. These
//!   GPIOs can be configured differently. The peripheral drivers will only
//!   support a single connection (that is, they disconnect previously
//!   configured signals on repeat calls to the same function), but you can use
//!   `esp_hal::gpio::OutputSignal::connect_to` (note that the type is currently
//!   hidden from the documentation) to connect multiple GPIOs to the same
//!   output signal.
//! - A GPIO input signal can be connected to any number of peripheral inputs.
//! - A GPIO output can be driven by only one peripheral output.

// 101
use crate::gpio::{
    self, AnyPin, InputPin, InputSignalType, Level, OutputPin, OutputSignalType, Pin, PinGuard,
    FUNC_IN_SEL_OFFSET, GPIO_FUNCTION, INPUT_SIGNAL_MAX, OUTPUT_SIGNAL_MAX,
};

// 102
use crate::peripherals::GPIO;

/// The base of all peripheral signals.
///
/// This trait represents a signal in the GPIO matrix. Signals are converted or
/// split from GPIO pins and can be connected to peripheral inputs and outputs.
///
/// All signals can be peripheral inputs, but not all output-like types should
/// be allowed to be passed as inputs. This trait bridges this gap by defining
/// the logic, but not declaring the signal to be an actual Input signal.
// 133
//pub trait PeripheralSignal<'d>: Sealed {
pub trait PeripheralSignal<'d> {
    /// Connects the peripheral input to an input signal source.
    fn connect_input_to_peripheral(&self, signal: gpio::InputSignal);
}

/// A signal that can be connected to a peripheral input.
///
/// Peripheral drivers are encouraged to accept types that implement this and
/// [`PeripheralOutput`] as arguments instead of pin types.
// 147
pub trait PeripheralInput<'d>: Into<InputSignal<'d>> + PeripheralSignal<'d> {}

/// A signal that can be connected to a peripheral input and/or output.
///
/// Peripheral drivers are encouraged to accept types that implement this and
/// [`PeripheralInput`] as arguments instead of pin types.
// 157
pub trait PeripheralOutput<'d>: Into<OutputSignal<'d>> + PeripheralSignal<'d> {
    /// Connects the peripheral output to an output signal target.
    // 160
    fn connect_peripheral_to_output(&self, signal: gpio::OutputSignal);

    /// Disconnects the peripheral output from an output signal target.
    ///
    /// This function clears the entry in the IO MUX that
    /// associates this output pin with a previously connected
    /// [signal](`gpio::OutputSignal`). Any other outputs connected to the
    /// peripheral remain intact.
    // 169
    fn disconnect_from_peripheral_output(&self);
}

// Pins
// 173
impl<'d, P> PeripheralSignal<'d> for P
where
    P: Pin + 'd,
{
    fn connect_input_to_peripheral(&self, signal: gpio::InputSignal) {
        let pin = unsafe { AnyPin::steal(self.number()) };
        InputSignal::new(pin).connect_input_to_peripheral(signal);
    }
}

// 182
impl<'d, P> PeripheralInput<'d> for P where P: InputPin + 'd {}

// 184
impl<'d, P> PeripheralOutput<'d> for P
where
    P: OutputPin + 'd,
{
    // 188
    fn connect_peripheral_to_output(&self, signal: gpio::OutputSignal) {
        let pin = unsafe { AnyPin::steal(self.number()) };
        OutputSignal::new(pin).connect_peripheral_to_output(signal);
    }

    // 192
    fn disconnect_from_peripheral_output(&self) {
        let pin = unsafe { AnyPin::steal(self.number()) };
        OutputSignal::new(pin).disconnect_from_peripheral_output();
    }
}

// Split signals
// 284
impl<'d> PeripheralSignal<'d> for InputSignal<'d> {
    fn connect_input_to_peripheral(&self, signal: gpio::InputSignal) {
        // Since there can only be one input signal connected to a peripheral
        // at a time, this function will disconnect any previously
        // connected input signals.
        self.pin.connect_to_peripheral_input(
            signal,
            self.is_input_inverted(),
            self.is_gpio_matrix_forced(),
        );
    }
}

// 298
impl<'d> PeripheralSignal<'d> for OutputSignal<'d> {
    fn connect_input_to_peripheral(&self, signal: gpio::InputSignal) {
        self.pin.connect_to_peripheral_input(
            signal,
            self.is_input_inverted(),
            self.is_gpio_matrix_forced(),
        );
    }
}

// 307
impl<'d> PeripheralOutput<'d> for OutputSignal<'d> {
    // 308
    fn connect_peripheral_to_output(&self, signal: gpio::OutputSignal) {
        self.pin.connect_peripheral_to_output(
            signal,
            self.is_output_inverted(),
            self.is_gpio_matrix_forced(),
            true,
            false,
        );
    }

    // 317
    fn disconnect_from_peripheral_output(&self) {
        self.pin.disconnect_from_peripheral_output();
    }
}

// 322
impl gpio::InputSignal {
    // 323
    fn can_use_gpio_matrix(self) -> bool {
        self as InputSignalType <= INPUT_SIGNAL_MAX
    }

    /// Connects a peripheral input signal to a GPIO or a constant level.
    ///
    /// Note that connecting multiple GPIOs to a single peripheral input is not
    /// possible and the previous connection will be replaced.
    ///
    /// Also note that a peripheral input must always be connected to something,
    /// so if you want to disconnect it from GPIOs, you should connect it to a
    /// constant level.
    ///
    /// This function allows connecting a peripheral input to either a
    /// [`PeripheralInput`] or [`PeripheralOutput`] implementation.
    #[inline]
    pub fn connect_to<'a>(self, pin: &impl PeripheralSignal<'a>) {
        pin.connect_input_to_peripheral(self);
    }
}

// 345
impl gpio::OutputSignal {
    // 346
    fn can_use_gpio_matrix(self) -> bool {
        self as OutputSignalType <= OUTPUT_SIGNAL_MAX
    }

    /// Connects a peripheral output signal to a GPIO.
    ///
    /// Note that connecting multiple output signals to a single GPIO is not
    /// possible and the previous connection will be replaced.
    ///
    /// Also note that it is possible to connect a peripheral output signal to
    /// multiple GPIOs, and old connections will not be cleared automatically.
    #[inline]
    // 359
    pub fn connect_to<'d>(self, pin: &impl PeripheralOutput<'d>) {
        pin.connect_peripheral_to_output(self);
    }

    /// Disconnects a peripheral output signal from a GPIO.
    #[inline]
    // 367
    pub fn disconnect_from<'d>(self, pin: &impl PeripheralOutput<'d>) {
        pin.disconnect_from_peripheral_output();
    }
}

// 371
enum Signal<'d> {
    Pin(AnyPin<'d>),
    _Level(Level),
}

// 375
impl Signal<'_> {
    // 376
    fn gpio_number(&self) -> Option<u8> {
        match &self {
            Signal::Pin(pin) => Some(pin.number()),
            Signal::_Level(_) => None,
        }
    }

    // 404
    fn connect_with_guard(self, signal: crate::gpio::OutputSignal) -> PinGuard {
        match self {
            Signal::Pin(pin) => PinGuard::new(pin, signal),
            Signal::_Level(_) => PinGuard::new_unconnected(signal),
        }
    }

    // 411
    fn connect_to_peripheral_input(
        &self,
        signal: gpio::InputSignal,
        is_inverted: bool,
        force_gpio: bool,
    ) {
        let use_gpio_matrix = match self {
            Signal::Pin(pin) => {
                let af = if is_inverted || force_gpio {
                    GPIO_FUNCTION
                } else {
                    //pin.input_signals(private::Internal)
                    pin.input_signals()
                        .iter()
                        .find(|(_af, s)| *s == signal)
                        .map(|(af, _)| *af)
                        .unwrap_or(GPIO_FUNCTION)
                };
                pin.set_alternate_function(af);
                af == GPIO_FUNCTION
            }
            Signal::_Level(_) => true,
        };

        let input = match self {
            Signal::Pin(pin) => pin.number(),
            Signal::_Level(Level::Low) => gpio::ZERO_INPUT,
            Signal::_Level(Level::High) => gpio::ONE_INPUT,
        };

        assert!(
            signal.can_use_gpio_matrix() || !use_gpio_matrix,
            "{:?} cannot be routed through the GPIO matrix",
            signal
        );
        // No need for a critical section, this is a write and not a modify operation.
        GPIO::regs()
            .func_in_sel_cfg(signal as usize - FUNC_IN_SEL_OFFSET)
            .write(|w| unsafe {
                // set this bit to bypass GPIO. 1:do not bypass GPIO. 0:bypass GPIO.
                w.sel().bit(use_gpio_matrix);
                // set this bit to invert input signal. 1:invert. 0:not invert.
                w.in_inv_sel().bit(is_inverted);
                // Connect to GPIO or constant level
                // set this value: s=0-53: connect GPIO[s] to this port.
                // s=0x38: set this port always high level. s=0x3C: set this port always low level.
                w.in_sel().bits(input)
            });
    }

    // 456
    fn connect_peripheral_to_output(
        &self,
        signal: gpio::OutputSignal,
        is_inverted: bool,
        force_gpio: bool,
        peripheral_control_output_enable: bool,
        invert_output_enable: bool,
    ) {
        let Signal::Pin(pin) = self else {
            return;
        };
        let af = if is_inverted || force_gpio {
            GPIO_FUNCTION
        } else {
            //pin.output_signals(private::Internal)
            pin.output_signals()
                .iter()
                .find(|(_af, s)| *s == signal)
                .map(|(af, _)| *af)
                .unwrap_or(GPIO_FUNCTION)
        };
        pin.set_alternate_function(af);

        let use_gpio_matrix = af == GPIO_FUNCTION;

        assert!(
            signal.can_use_gpio_matrix() || use_gpio_matrix,
            "{:?} cannot be routed through the GPIO matrix",
            signal
        );

        GPIO::regs()
            .func_out_sel_cfg(pin.number() as usize)
            .write(|w| unsafe {
                if use_gpio_matrix {
                    // Ignored if the signal is not routed through the GPIO matrix - alternate
                    // function selects peripheral signal directly.
                    w.out_sel().bits(signal as _);
                    w.inv_sel().bit(is_inverted);
                }
                // set this bit to select output enable signal.
                // 1:use GPIO_ENABLE_REG[n] as output enable signal.
                // 0:use peripheral output enable signal.
                w.oen_sel().bit(!peripheral_control_output_enable);
                w.oen_inv_sel().bit(invert_output_enable)
            });
    }

    // 500
    fn disconnect_from_peripheral_output(&self) {
        let Some(number) = self.gpio_number() else {
            return;
        };
        GPIO::regs()
            .func_out_sel_cfg(number as usize)
            .modify(|_, w| unsafe { w.out_sel().bits(gpio::OutputSignal::GPIO as _) });
    }
}

// 510
bitflags::bitflags! {
    #[derive(Clone, Copy)]
    struct InputFlags: u8 {
        const ForceGpioMatrix = 1 << 0;
        const Frozen          = 1 << 1;
        const InvertInput     = 1 << 2;
    }
}

/// An input signal between a peripheral and a GPIO pin.
///
/// If the `InputSignal` was obtained from a pin driver such as
/// [`Input`](crate::gpio::Input::split), the GPIO driver will be responsible
/// for configuring the pin with the correct settings, peripheral drivers will
/// not be able to modify the pin settings.
///
/// Multiple input signals can be connected to one pin.
// 528
pub struct InputSignal<'d> {
    pin: Signal<'d>,
    flags: InputFlags,
}

// 545
impl<'d, P> From<P> for InputSignal<'d>
where
    P: Pin + 'd,
{
    fn from(input: P) -> Self {
        InputSignal::new(input.degrade())
    }
}

// 578
impl<'d> InputSignal<'d> {
    // 579
    fn new_inner(inner: Signal<'d>) -> Self {
        Self {
            pin: inner,
            flags: InputFlags::empty(),
        }
    }

    // 586
    pub(crate) fn new(pin: AnyPin<'d>) -> Self {
        Self::new_inner(Signal::Pin(pin))
    }

    /// Returns `true` if the input signal is configured to be inverted.
    ///
    /// Note that the hardware is not configured until the signal is actually
    /// connected to a peripheral.
    // 644
    pub fn is_input_inverted(&self) -> bool {
        self.flags.contains(InputFlags::InvertInput)
    }

    /// Consumes the signal and returns a new one that forces the GPIO matrix
    /// to be used.
    // 657
    pub fn with_gpio_matrix_forced(mut self, force: bool) -> Self {
        self.flags.set(InputFlags::ForceGpioMatrix, force);
        self
    }

    /// Returns `true` if the input signal must be routed through the GPIO
    /// matrix.
    // 664
    pub fn is_gpio_matrix_forced(&self) -> bool {
        self.flags.contains(InputFlags::ForceGpioMatrix)
    }

    delegate::delegate! {
        to match &self.pin {
            Signal::Pin(_) if self.flags.contains(InputFlags::Frozen) => NoOp,
            Signal::Pin(signal) => signal,
            Signal::_Level(_) => NoOp,
        } {
            pub fn apply_input_config(&self, _config: &gpio::InputConfig);
            pub fn set_input_enable(&self, on: bool);
        }
    }
}

// 694
bitflags::bitflags! {
    #[derive(Clone, Copy)]
    struct OutputFlags: u8 {
        const ForceGpioMatrix = 1 << 0;
        const Frozen          = 1 << 1;
        const InvertInput     = 1 << 2;
        const InvertOutput    = 1 << 3;
    }
}

/// An (input and) output signal between a peripheral and a GPIO pin.
///
/// If the `OutputSignal` was obtained from a pin driver such as
/// [`Output`](crate::gpio::Output::split), the GPIO driver will be responsible
/// for configuring the pin with the correct settings, peripheral drivers will
/// not be able to modify the pin settings.
///
/// Note that connecting this to a peripheral input will enable the input stage
/// of the GPIO pin.
///
/// Multiple pins can be connected to one output signal.
// 716
pub struct OutputSignal<'d> {
    pin: Signal<'d>,
    flags: OutputFlags,
}

// 734
impl<'d, P> From<P> for OutputSignal<'d>
where
    P: OutputPin + 'd,
{
    fn from(output: P) -> Self {
        OutputSignal::new(output.degrade())
    }
}

// 756
impl<'d> OutputSignal<'d> {
    // 757
    fn new_inner(inner: Signal<'d>) -> Self {
        Self {
            pin: inner,
            flags: OutputFlags::empty(),
        }
    }

    // 764
    pub(crate) fn new(pin: AnyPin<'d>) -> Self {
        Self::new_inner(Signal::Pin(pin))
    }

    /// Returns `true` if the input signal is configured to be inverted.
    ///
    /// Note that the hardware is not configured until the signal is actually
    /// connected to a peripheral.
    // 808
    pub fn is_input_inverted(&self) -> bool {
        self.flags.contains(OutputFlags::InvertInput)
    }

    /// Returns `true` if the output signal is configured to be inverted.
    ///
    /// Note that the hardware is not configured until the signal is actually
    /// connected to a peripheral.
    // 816
    pub fn is_output_inverted(&self) -> bool {
        self.flags.contains(OutputFlags::InvertOutput)
    }

    /// Consumes the signal and returns a new one that forces the GPIO matrix
    /// to be used.
    // 834
    pub fn with_gpio_matrix_forced(mut self, force: bool) -> Self {
        self.flags.set(OutputFlags::ForceGpioMatrix, force);
        self
    }

    /// Returns `true` if the input signal must be routed through the GPIO
    /// matrix.
    // 843
    pub fn is_gpio_matrix_forced(&self) -> bool {
        self.flags.contains(OutputFlags::ForceGpioMatrix)
    }

    pub(crate) fn connect_with_guard(self, signal: crate::gpio::OutputSignal) -> PinGuard {
        signal.connect_to(&self);
        self.pin.connect_with_guard(signal)
    }

    // 881
    delegate::delegate! {
        to match &self.pin {
            Signal::Pin(_) if self.flags.contains(OutputFlags::Frozen) => NoOp,
            Signal::Pin(pin) => pin,
            Signal::_Level(_) => NoOp,
        } {
            pub fn apply_output_config(&self, _config: &gpio::OutputConfig);
            pub fn set_output_enable(&self, on: bool);
            pub fn set_output_high(&self, on: bool);
        }
    }
}

// 898
struct NoOp;

// 900
impl NoOp {
    fn set_input_enable(&self, _on: bool) {}
    fn set_output_enable(&self, _on: bool) {}
    fn set_output_high(&self, _on: bool) {}
    fn apply_input_config(&self, _config: &gpio::InputConfig) {}
    fn apply_output_config(&self, _config: &gpio::OutputConfig) {}
}
