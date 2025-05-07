use proc_macro::TokenStream;
use syn::{parse, Item};

// 16
pub fn ram(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item: Item = parse(input).expect("failed to parse input");
    let section = quote::quote! {
        #[unsafe(link_section = ".rwtext")]
        #[inline(never)] // make certain function is not inlined
    };

    let output = quote::quote! {
        #section
        #item
    };

    output.into()
}
