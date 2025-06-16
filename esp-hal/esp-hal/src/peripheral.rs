//! # Exclusive peripheral access

/// Creates a new `Peripherals` struct and its associated methods.
///
/// The macro has a few fields doing different things, in the form of
/// `second <= third (fourth)`.
/// - The second field is the name of the peripheral, as it appears in the
///   `Peripherals` struct.
/// - The third field is the name of the peripheral as it appears in the PAC.
///   This may be `virtual` if the peripheral is not present in the PAC.
/// - The fourth field is an optional list of interrupts that can be bound to
///   the peripheral.

#[macro_export]
// 15
macro_rules! peripherals {
        (
            peripherals: [
                $(
                    $name:ident <= $from_pac:tt $(($($interrupt:ident),*))?
                ),* $(,)?
            ],
        ) => {
            // 31
            paste::paste! {
                $(
                    $crate::create_peripheral!($name <= $from_pac);
                )*

                /// The `Peripherals` struct provides access to all of the hardware peripherals on the chip.
                #[allow(non_snake_case)]
                // 44
                pub struct Peripherals {
                    $(
                        #[doc = concat!("The ", stringify!($name), " peripheral.")]
                        pub $name: $name<'static>,
                    )*
                }

                // 73
                impl Peripherals {
                    /// Returns all the peripherals *once*
                    #[inline]
                    // 76
                    pub(crate) fn take() -> Self {
                        #[unsafe(no_mangle)]
                        static mut _ESP_HAL_DEVICE_PERIPHERALS: bool = false;

                        critical_section::with(|_|
                        unsafe {
                            if _ESP_HAL_DEVICE_PERIPHERALS {
                                panic!("init called more than once!")
                            }
                            _ESP_HAL_DEVICE_PERIPHERALS = true;
                            Self::steal()
                        })
                    }

                    /// Unsafely create an instance of this peripheral out of thin air.
                    ///
                    /// # Safety
                    ///
                    /// You must ensure that you're only using one instance of this type at a time.
                    #[inline]
                    // 95
                    pub unsafe fn steal() -> Self {
                        unsafe {
                            Self {
                                $(
                                    $name: $name::steal(),
                                )*
                            }
                        }
                    }
                }
            }
        };
    }

#[macro_export]
/// Macro to create a peripheral structure.
// 155
macro_rules! create_peripheral {
        // 156
        ($(#[$attr:meta])? $name:ident <= virtual) => {
            $(#[$attr])?
            #[derive(Debug)]
            #[cfg_attr(feature = "defmt", derive(defmt::Format))]
            #[non_exhaustive]
            #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
            #[doc = concat!(stringify!($name), " peripheral singleton")]
            pub struct $name<'a> {
                _marker: core::marker::PhantomData<&'a mut ()>,
            }

            // 167
            impl $name<'_> {
                /// Unsafely create an instance of this peripheral out of thin air.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of this type at a time.
                #[inline]
                // 174
                pub unsafe fn steal() -> Self {
                    Self {
                        _marker: core::marker::PhantomData,
                    }
                }

                /// Unsafely clone this peripheral reference.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of this type at a time.
                #[inline]
                #[allow(dead_code)]
                // 187
                pub unsafe fn clone_unchecked(&self) -> Self {
                    unsafe { Self::steal() }
                }

                /// Creates a new peripheral reference with a shorter lifetime.
                ///
                /// Use this method if you would like to keep working with the peripheral after
                /// you dropped the driver that consumes this.
                #[inline]
                #[allow(dead_code)]
                // 197
                pub fn reborrow(&mut self) -> $name<'_> {
                    unsafe { self.clone_unchecked() }
                }
            }

            //impl $crate::private::Sealed for $name<'_> {}
        };

        // 205
        ($(#[$attr:meta])? $name:ident <= $base:ident) => {
            $crate::create_peripheral!($(#[$attr])? $name <= virtual);

            // 208
            impl $name<'_> {
                /// Pointer to the register block
                // 211
                pub const PTR: *const <pac::$base as core::ops::Deref>::Target = pac::$base::PTR;

                /// Return the pointer to the register block
                #[inline(always)]
                // 216
                pub const fn ptr() -> *const <pac::$base as core::ops::Deref>::Target {
                    pac::$base::PTR
                }

                /// Return a reference to the register block
                #[inline(always)]
                // 223
                pub const fn regs<'a>() -> &'a <pac::$base as core::ops::Deref>::Target {
                    unsafe { &*Self::PTR }
                }

                /// Return a reference to the register block
                #[inline(always)]
                // 230
                pub fn register_block(&self) -> &<pac::$base as core::ops::Deref>::Target {
                    unsafe { &*Self::PTR }
                }
            }
        };
    }
