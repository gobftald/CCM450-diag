// 47
use proc_macro::TokenStream;

// 49
mod blocking;

#[cfg(feature = "embassy")]
// 52
mod embassy;

// 61
mod ram;

#[proc_macro_attribute]
#[proc_macro_error2::proc_macro_error]
// 119
pub fn ram(args: TokenStream, input: TokenStream) -> TokenStream {
    ram::ram(args, input)
}

#[cfg(feature = "embassy")]
#[proc_macro_attribute]
// 179
pub fn embassy_main(args: TokenStream, item: TokenStream) -> TokenStream {
    embassy::main(args, item)
}

#[proc_macro_attribute]
// 211
pub fn blocking_main(args: TokenStream, input: TokenStream) -> TokenStream {
    blocking::main(args, input)
}
