// 47
use proc_macro::TokenStream;

// 49
mod blocking;

#[cfg(feature = "embassy")]
// 52
mod embassy;
mod interrupt;

// 61
mod ram;

#[proc_macro_attribute]
#[proc_macro_error2::proc_macro_error]
// 119
pub fn ram(args: TokenStream, input: TokenStream) -> TokenStream {
    ram::ram(args, input)
}

/// Mark a function as an interrupt handler.
///
/// Optionally a priority can be specified, e.g. `#[handler(priority =
/// esp_hal::interrupt::Priority::Priority2)]`.
///
/// If no priority is given, `Priority::min()` is assumed
#[proc_macro_attribute]
#[proc_macro_error2::proc_macro_error]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    interrupt::handler(args, input)
}

#[cfg(feature = "embassy")]
#[proc_macro_attribute]
// 179
pub fn embassy_main(args: TokenStream, item: TokenStream) -> TokenStream {
    embassy::main(args, item)
}

#[proc_macro_attribute]
// 208
pub fn blocking_main(args: TokenStream, input: TokenStream) -> TokenStream {
    blocking::main(args, input)
}
