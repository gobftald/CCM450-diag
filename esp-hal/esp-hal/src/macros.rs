//#[doc(hidden)]
/// Shorthand to define AnyPeripheral instances.
///
/// This macro generates the following:
///
/// - An `AnyPeripheral` struct, name provided by the macro call.
/// - An `AnyPeripheralInner` enum, with the same variants as the original
///   peripheral.
/// - A `From` implementation for each peripheral variant.
/// - A `degrade` method for each peripheral variant using the
///   `IntoAnyPeripheral` trait.
#[macro_export]
// 98
macro_rules! any_peripheral {
    ($(#[$meta:meta])* $vis:vis peripheral $name:ident<'d> {
        $(
            $(#[cfg($variant_meta:meta)])*
            $variant:ident($inner:ty)
        ),* $(,)?
    }) => {
        /*
        // 105
        paste::paste! {
            $(#[$meta])*
            ///
            /// This struct is a type-erased version of a peripheral singleton. It is useful
            /// for creating arrays of peripherals, or avoiding generics. Peripheral singletons
            /// can be type erased by using their `From` implementation.
            ///
            #[derive(Debug)]
            #[cfg_attr(feature = "defmt", derive(defmt::Format))]
            // 117
            $vis struct $name<'d>([< $name Inner >]<'d>);

            // 119
            impl $name<'_> {
                /// Unsafely clone this peripheral reference.
                ///
                /// # Safety
                ///
                /// You must ensure that you're only using one instance of this type at a time.
                #[inline]
                pub unsafe fn clone_unchecked(&self) -> Self { unsafe {
                    match &self.0 {
                        $(
                            $(#[cfg($variant_meta)])*
                            [< $name Inner >]::$variant(inner) => $name([<$name Inner>]::$variant(inner.clone_unchecked())),
                        )*
                    }
                }}

                /// Creates a new peripheral reference with a shorter lifetime.
                ///
                /// Use this method if you would like to keep working with the peripheral after
                /// you dropped the driver that consumes this.
                #[inline]
                pub fn reborrow(&mut self) -> $name<'_> {
                    unsafe { self.clone_unchecked() }
                }
            }

            $(#[$meta])*
            #[derive(Debug)]
            // 149
            enum [< $name Inner >]<'d> {
                $(
                    $(#[cfg($variant_meta)])*
                    $variant($inner),
                )*
            }

            #[cfg(feature = "defmt")]
            // 157
            impl defmt::Format for [< $name Inner >]<'_> {
                fn format(&self, fmt: defmt::Formatter<'_>) {
                    match self {
                        $(
                            $(#[cfg($variant_meta)])*
                            [< $name Inner >]::$variant(inner) => inner.format(fmt),
                        )*
                    }
                }
            }

            // Trick to make peripherals implement Into, without
            // requiring Instance traits to have lifetimes.
            // 171
            //pub trait [<Into $name>]: Sized + $crate::private::Sealed {
            pub trait [<Into $name>]: Sized {
                fn degrade<'a>(self) -> $name<'a>
                where
                    Self: 'a;
            }

            // AnyPeripheral converts into itself
            // 178
            impl<'d> [<Into $name>] for $name<'d> {
                #[inline]
                fn degrade<'a>(self) -> $name<'a>
                where
                    Self: 'a,
                {
                    self
                }
            }

            $(
                // Variants convert into AnyPeripheral
                $(#[cfg($variant_meta)])*
                // 191
                impl<'d> [<Into $name>] for $inner {
                    #[inline]
                    fn degrade<'a>(self) -> $name<'a>
                    where
                        Self: 'a,
                    {
                        $name::from(self)
                    }
                }

                // 201
                $(#[cfg($variant_meta)])*
                // 202
                impl<'d> From<$inner> for $name<'d> {
                    #[inline]
                    fn from(inner: $inner) -> Self {
                        Self([< $name Inner >]::$variant(inner))
                    }
                }
            )*

        }
        */

        #[doc = concat!("Utilities related to [`", stringify!($name), "`]")]
        #[doc(hidden)]
        //#[instability::unstable]
        pub mod any {
            #[allow(unused_imports)]
            use super::*;

            macro_rules! delegate {
                ($any:ident, $inner_ident:ident => $code:tt) => {
                    match &$any.0 {
                        $(
                            $(#[cfg($variant_meta)])*
                            any::Inner::$variant($inner_ident) => $code,
                        )*
                    }
                }
            }

            pub(crate) use delegate;

            $(#[$meta])*
            #[derive(Debug)]
            pub(crate) enum Inner<'d> {
                $(
                    $(#[cfg($variant_meta)])*
                    $variant($inner),
                )*
            }

            #[cfg(feature = "defmt")]
            impl defmt::Format for Inner<'_> {
                fn format(&self, fmt: defmt::Formatter<'_>) {
                    match self {
                        $(
                            $(#[cfg($variant_meta)])*
                            Self::$variant(inner) => inner.format(fmt),
                        )*
                    }
                }
            }

            // Trick to make peripherals implement something Into-like, without
            // requiring Instance traits to have lifetimes. Rustdoc will list
            // this trait as a supertrait, but will not give its definition.
            // Users are encouraged to use From to convert a singleton into its
            // relevant AnyPeripheral counterpart.
            #[allow(unused)]
            //pub trait Degrade: Sized + $crate::private::Sealed {
            pub trait Degrade: Sized {
                fn degrade<'a>(self) -> super::$name<'a>
                where
                    Self: 'a;
            }
        }

        $(#[$meta])*
        ///
        /// This struct is a type-erased version of a peripheral singleton. It is useful
        /// for creating arrays of peripherals, or avoiding generics. Peripheral singletons
        /// can be type erased by using their `From` implementation.
        ///
        /// ```rust,ignore
        #[doc = concat!("let any_peripheral = ", stringify!($name), "::from(peripheral);")]
        /// ```
        #[derive(Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        $vis struct $name<'d>(any::Inner<'d>);

        impl $name<'_> {
            /// Unsafely clone this peripheral reference.
            ///
            /// # Safety
            ///
            /// You must ensure that you're only using one instance of this type at a time.
            #[inline]
            pub unsafe fn clone_unchecked(&self) -> Self { unsafe {
                any::delegate!(self, inner => { Self::from(inner.clone_unchecked()) })
            }}

            /// Creates a new peripheral reference with a shorter lifetime.
            ///
            /// Use this method if you would like to keep working with the peripheral after
            /// you dropped the driver that consumes this.
            ///
            /// See [Peripheral singleton] section for more information.
            ///
            /// [Peripheral singleton]: crate#peripheral-singletons
            #[inline]
            pub fn reborrow(&mut self) -> $name<'_> {
                unsafe { self.clone_unchecked() }
            }

            //#[procmacros::doc_replace]
            /// Attempts to downcast the pin into the underlying peripheral instance.
            ///
            /// ## Example
            ///
            /// ```rust,no_run
            /// # {before_snippet}
            /// #
            /// # use esp_hal::{
            /// #     uart::AnyUart as AnyPeripheral,
            /// #     peripherals::{UART0 as PERI0, UART1 as PERI1},
            /// # };
            /// #
            /// # let peri0 = peripherals.UART0;
            /// # let peri1 = peripherals.UART1;
            /// // let peri0 = peripherals.PERI0;
            /// // let peri1 = peripherals.PERI1;
            /// let any_peri0 = AnyPeripheral::from(peri0);
            /// let any_peri1 = AnyPeripheral::from(peri1);
            ///
            /// let uart0 = any_peri0
            ///     .downcast::<PERI0>()
            ///     .expect("This downcast succeeds because AnyPeripheral was created from Peri0");
            /// let uart0 = any_peri1
            ///     .downcast::<PERI0>()
            ///     .expect_err("This AnyPeripheral was created from Peri1, it cannot be downcast to Peri0");
            /// #
            /// # {after_snippet}
            /// ```
            #[inline]
            pub fn downcast<P>(self) -> Result<P, Self>
            where
                Self: TryInto<P, Error = Self>
            {
                self.try_into()
            }
        }

        // AnyPeripheral converts into itself
        impl<'d> any::Degrade for $name<'d> {
            #[inline]
            fn degrade<'a>(self) -> $name<'a>
            where
                Self: 'a,
            {
                self
            }
        }

        $(
            // Variants convert into AnyPeripheral
            $(#[cfg($variant_meta)])*
            impl<'d> any::Degrade for $inner {
                #[inline]
                fn degrade<'a>(self) -> $name<'a>
                where
                    Self: 'a,
                {
                    $name::from(self)
                }
            }

            $(#[cfg($variant_meta)])*
            impl<'d> From<$inner> for $name<'d> {
                #[inline]
                fn from(inner: $inner) -> Self {
                    Self(any::Inner::$variant(inner))
                }
            }

            $(#[cfg($variant_meta)])*
            impl<'d> TryFrom<$name<'d>> for $inner {
                type Error = $name<'d>;

                #[inline]
                fn try_from(any: $name<'d>) -> Result<Self, $name<'d>> {
                    #[allow(irrefutable_let_patterns)]
                    if let $name(any::Inner::$variant(inner)) = any {
                        Ok(inner)
                    } else {
                        Err(any)
                    }
                }
            }
        )*
    };
}

/// Macro to ignore tokens.
///
/// This is useful when we need existence of a metavariable (to expand a
/// repetition), but we don't need to use it.
#[macro_export]
#[doc(hidden)]
macro_rules! ignore {
    ($($item:tt)*) => {};
}
