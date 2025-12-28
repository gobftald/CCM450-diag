//! # Peripheral Instances
//!
//! This module creates singleton instances for each of the various peripherals,
//! and re-exports them to allow users to access and use them in their
//! applications.
//!
//! Should be noted that that the module also re-exports the [Interrupt] enum
//! from the PAC, allowing users to handle interrupts associated with these
//! peripherals.

// We need to export this for users to use
#[doc(hidden)]
pub use pac::Interrupt;

pub(crate) use crate::soc::pac;

#[macro_export]
/// Macro to create a peripheral structure.
macro_rules! create_peripheral {
    ($(#[$attr:meta])? $name:ident <= virtual ($($interrupt:ident: { $bind:ident, $enable:ident, $disable:ident }),*)) => {
        $(#[$attr])?
        #[derive(Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        #[non_exhaustive]
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[doc = concat!(stringify!($name), " peripheral singleton")]
        pub struct $name<'a> {
            _marker: core::marker::PhantomData<&'a mut ()>,
        }

        impl $name<'_> {
            /// Unsafely create an instance of this peripheral out of thin air.
            ///
            /// # Safety
            ///
            /// You must ensure that you're only using one instance of this type at a time.
            #[inline]
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
            pub unsafe fn clone_unchecked(&self) -> Self {
                unsafe { Self::steal() }
            }

            /// Creates a new peripheral reference with a shorter lifetime.
            ///
            /// Use this method if you would like to keep working with the peripheral after
            /// you dropped the driver that consumes this.
            #[inline]
            #[allow(dead_code)]
            pub fn reborrow(&mut self) -> $name<'_> {
                unsafe { self.clone_unchecked() }
            }
        }

        //impl $crate::private::Sealed for $name<'_> {}
    };

    ($(#[$attr:meta])? $name:ident <= $base:ident $interrupts:tt) => {
        create_peripheral!($(#[$attr])? $name <= virtual $interrupts);

        impl $name<'_> {
            /// Pointer to the register block
            pub const PTR: *const <pac::$base as core::ops::Deref>::Target = pac::$base::PTR;

            /// Return the pointer to the register block
            #[inline(always)]
            pub const fn ptr() -> *const <pac::$base as core::ops::Deref>::Target {
                pac::$base::PTR
            }

            /// Return a reference to the register block
            #[inline(always)]
            pub const fn regs<'a>() -> &'a <pac::$base as core::ops::Deref>::Target {
                unsafe { &*Self::PTR }
            }

            /// Return a reference to the register block
            #[inline(always)]
            pub fn register_block(&self) -> &<pac::$base as core::ops::Deref>::Target {
                unsafe { &*Self::PTR }
            }
        }
    };
}

for_each_peripheral! {
    // Define stable peripheral singletons
    ($name:ident <= $from_pac:tt $interrupts:tt) => {
        create_peripheral!($name <= $from_pac $interrupts);
    };

    // Define unstable peripheral singletons
    ($name:ident <= $from_pac:tt $interrupts:tt (unstable)) => {
        create_peripheral!(/*#[instability::unstable]*/ $name <= $from_pac $interrupts);
    };

    // Define the Peripherals struct
    //(singletons $( ($name:ident $(($unstable:ident))?) ),*) => {
    (all $( ($name:ident <= $from_pac:tt $interrupts:tt $(($unstable:ident))?) ),*) => {
        /// The `Peripherals` struct provides access to all of the hardware peripherals on the chip.
        #[allow(non_snake_case)]
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
            pub(crate) fn take() -> Self {
                #[unsafe(no_mangle)]
                static mut _ESP_HAL_DEVICE_PERIPHERALS: bool = false;

                crate::ESP_HAL_LOCK.lock(||
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

        /*
        $crate::gpio! {
            $( ($pin, $($pin_tokens)* ) )*
        }
        */
    };
}
