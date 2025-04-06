use proc_macro::TokenStream;

// 52
mod blocking_main;

#[proc_macro_attribute]
// 211
pub fn blocking_main(args: TokenStream, input: TokenStream) -> TokenStream {
    blocking_main::main(args, input)
}
