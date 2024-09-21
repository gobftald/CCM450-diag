/// Trait for any type that can be used as a peripheral of type `P`.
///
/// This is used in driver constructors, to allow passing either owned
/// peripherals (e.g. `UART0`), or borrowed peripherals (e.g. `&mut UART0`).
///
/// For example, if you have a driver with a constructor like this:
///
/// ```rust, ignore
/// impl<'d, T> Uart<'d, T, Blocking> {
///     pub fn new<TX: PeripheralOutput, RX: PeripheralInput>(
///         uart: impl Peripheral<P = T> + 'd,
///         rx: impl Peripheral<P = RX> + 'd,
///         tx: impl Peripheral<P = TX> + 'd,
///     ) -> Result<Self, Error> {
///         Ok(Self { .. })
///     }
/// }
/// ```
///
/// You may call it with owned peripherals, which yields an instance that can
/// live forever (`'static`):
///
/// ```rust, ignore
/// let mut uart: Uart<'static, ...> = Uart::new(p.UART0, pins.gpio0, pins.gpio1);
/// ```
///
/// Or you may call it with borrowed peripherals, which yields an instance that
/// can only live for as long as the borrows last:
///
/// ```rust, ignore
/// let mut uart: Uart<'_, ...> = Uart::new(&mut p.UART0, &mut pins.gpio0, &mut pins.gpio1);
/// ```
///
/// # Implementation details, for HAL authors
///
/// When writing a HAL, the intended way to use this trait is to take `impl
/// Peripheral<P = ..>` in the HAL's public API (such as driver constructors),
/// calling `.into_ref()` to obtain a `PeripheralRef`, and storing that in the
/// driver struct.
///
/// `.into_ref()` on an owned `T` yields a `PeripheralRef<'static, T>`.
/// `.into_ref()` on an `&'a mut T` yields a `PeripheralRef<'a, T>`.
// 125
pub trait Peripheral: Sized + crate::private::Sealed {
    type P;

    /// Unsafely clone (duplicate) a peripheral singleton.
    ///
    /// # Safety
    ///
    /// This returns an owned clone of the peripheral. You must manually ensure
    /// only one copy of the peripheral is in use at a time. For example, don't
    /// create two SPI drivers on `SPI1`, because they will "fight" each other.
    ///
    /// You should strongly prefer using `into_ref()` instead. It returns a
    /// `PeripheralRef`, which allows the borrow checker to enforce this at
    /// compile time.
    unsafe fn clone_unchecked(&mut self) -> Self::P;
}

// 168
mod peripheral_macros {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! peripherals {
        ($($(#[$cfg:meta])? $name:ident <= $from_pac:tt $(($($interrupt:ident),*))? ),*$(,)?) => {

            /// Contains the generated peripherals which implement [`Peripheral`]
            mod peripherals {
                $(
                    $crate::create_peripheral!($(#[$cfg])? $name <= $from_pac);
                )*
            }

            /// The `Peripherals` struct provides access to all of the hardware peripherals on the chip.
            #[allow(non_snake_case)]
            pub struct Peripherals {
                $(
                    $(#[$cfg])?
                    /// Each field represents a hardware peripheral.
                    pub $name: peripherals::$name,
                )*
            }

            /* temporarily to suppress warning
            impl Peripherals {
                /// Returns all the peripherals *once*
                #[inline]
                pub(crate) fn take() -> Self {
                    #[no_mangle]
                    static mut _ESP_HAL_DEVICE_PERIPHERALS: bool = false;

                    critical_section::with(|_| unsafe {
                        if _ESP_HAL_DEVICE_PERIPHERALS {
                            panic!("init called more than once!")
                        }
                        _ESP_HAL_DEVICE_PERIPHERALS = true;
                        Self::steal()
                    })
                }
            }
            */

            impl Peripherals {
                /// Unsafely create an instance of this peripheral out of thin air.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of this type at a time.
                #[inline]
                pub unsafe fn steal() -> Self {
                    Self {
                        $(
                            $(#[$cfg])?
                            $name: peripherals::$name::steal(),
                        )*
                    }
                }
            }

            // expose the new structs
            $(
                pub use peripherals::$name;
            )*

            $(
                $(
                    impl peripherals::$name {
                        $(
                            paste::paste!{
                                /// Binds an interrupt handler to the corresponding interrupt for this peripheral.
                                pub fn [<bind_ $interrupt:lower _interrupt >](&mut self, handler: unsafe extern "C" fn() -> ()) {
                                    unsafe { $crate::interrupt::bind_interrupt($crate::peripherals::Interrupt::$interrupt, handler); }
                                }
                            }
                        )*
                    }
                )*
            )*

        }
    }

    #[doc(hidden)]
    #[macro_export]
    /// Macro to create a peripheral structure.
    // 316
    macro_rules! create_peripheral {
        ($(#[$cfg:meta])? $name:ident <= virtual) => {
            $(#[$cfg])?
            #[derive(Debug)]
            #[allow(non_camel_case_types)]
            /// Represents a virtual peripheral with no associated hardware.
            ///
            /// This struct is generated by the `create_peripheral!` macro when the peripheral
            /// is defined as virtual.
            pub struct $name { _inner: () }

            $(#[$cfg])?
            impl $name {
                /// Unsafely create an instance of this peripheral out of thin air.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of this type at a time.
                #[inline]
                pub unsafe fn steal() -> Self {
                    Self { _inner: () }
                }
            }

            impl $crate::peripheral::Peripheral for $name {
                type P = $name;

                #[inline]
                unsafe fn clone_unchecked(&mut self) -> Self::P {
                    Self::steal()
                }
            }

            impl $crate::private::Sealed for $name {}
        };
        ($(#[$cfg:meta])? $name:ident <= $base:ident) => {
            $(#[$cfg])?
            #[derive(Debug)]
            #[allow(non_camel_case_types)]
            /// Represents a concrete hardware peripheral.
            ///
            /// This struct is generated by the `create_peripheral!` macro when the peripheral
            /// is tied to an actual hardware device.
            pub struct $name { _inner: () }

            $(#[$cfg])?
            impl $name {
                /// Unsafely create an instance of this peripheral out of thin air.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of this type at a time.
                #[inline]
                pub unsafe fn steal() -> Self {
                    Self { _inner: () }
                }

                /// Pointer to the register block
                pub const PTR: *const <super::pac::$base as core::ops::Deref>::Target = super::pac::$base::PTR;

                /// Return the pointer to the register block
                #[inline(always)]
                pub const fn ptr() -> *const <super::pac::$base as core::ops::Deref>::Target {
                    super::pac::$base::PTR
                }
            }

            impl core::ops::Deref for $name {
                type Target = <super::pac::$base as core::ops::Deref>::Target;

                fn deref(&self) -> &Self::Target {
                    unsafe { &*Self::PTR }
                }
            }

            impl core::ops::DerefMut for $name {

                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *(Self::PTR as *mut _)  }
                }
            }

            impl $crate::peripheral::Peripheral for $name {
                type P = $name;

                #[inline]
                unsafe fn clone_unchecked(&mut self) -> Self::P {
                    Self::steal()
                }
            }

            impl $crate::private::Sealed for $name {}
        };
    }
}
