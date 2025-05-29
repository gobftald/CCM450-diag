#[doc(hidden)]
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

            $(#[$meta])*
            #[derive(Debug)]
            // 149
            enum [< $name Inner >]<'d> {
                $(
                    $(#[cfg($variant_meta)])*
                    $variant($inner),
                )*
            }

            $(
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
    };
}
