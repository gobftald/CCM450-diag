// 47
use proc_macro::TokenStream;

// 49
mod blocking;

#[cfg(feature = "embassy")]
// 52
mod embassy;

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
