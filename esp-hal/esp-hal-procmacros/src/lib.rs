use proc_macro::TokenStream;

#[cfg(feature = "embassy")]
mod embassy;

#[cfg(feature = "ram")]
#[derive(Debug, Default, darling::FromMeta)]
#[darling(default)]
// 68
struct RamArgs {
    rtc_fast: bool,
    rtc_slow: bool,
    persistent: bool,
    zeroed: bool,
}

#[cfg(feature = "ram")]
#[proc_macro_attribute]
#[proc_macro_error2::proc_macro_error]
// 132
pub fn ram(args: TokenStream, input: TokenStream) -> TokenStream {
    use darling::{ast::NestedMeta, Error, FromMeta};
    use proc_macro::Span;
    use proc_macro_error2::abort;
    use syn::{parse, Item};

    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let RamArgs {
        rtc_fast,
        rtc_slow,
        persistent,
        zeroed,
    } = match FromMeta::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let item: Item = parse(input).expect("failed to parse input");

    /*
    #[cfg(not(feature = "rtc_slow"))]
    if rtc_slow {
        abort!(
            Span::call_site(),
            "rtc_slow is not available for this target"
        );
    }
    */

    let is_fn = matches!(item, Item::Fn(_));
    let section_name = match (is_fn, rtc_fast, rtc_slow, persistent, zeroed) {
        (true, false, false, false, false) => Ok(".rwtext"),
        /*
        (true, true, false, false, false) => Ok(".rtc_fast.text"),
        (true, false, true, false, false) => Ok(".rtc_slow.text"),
        */
        (false, false, false, false, false) => Ok(".data"),

        /*
        (false, true, false, false, false) => Ok(".rtc_fast.data"),
        (false, true, false, true, false) => Ok(".rtc_fast.persistent"),
        (false, true, false, false, true) => Ok(".rtc_fast.bss"),

        (false, false, true, false, false) => Ok(".rtc_slow.data"),
        (false, false, true, true, false) => Ok(".rtc_slow.persistent"),
        (false, false, true, false, true) => Ok(".rtc_slow.bss"),
        */
        _ => Err(()),
    };

    let section = match (is_fn, section_name) {
        (true, Ok(section_name)) => quote::quote! {
            #[link_section = #section_name]
            #[inline(never)] // make certain function is not inlined
        },
        (false, Ok(section_name)) => quote::quote! {
            #[link_section = #section_name]
        },
        (_, Err(_)) => {
            abort!(Span::call_site(), "Invalid combination of ram arguments");
        }
    };

    let trait_check = if zeroed {
        Some("zeroable")
    } else if persistent {
        Some("persistable")
    } else {
        None
    };
    let trait_check = trait_check.map(|name| {
        use proc_macro_crate::{crate_name, FoundCrate};

        let hal = proc_macro2::Ident::new(
            if let Ok(FoundCrate::Name(ref name)) = crate_name("esp-hal") {
                name
            } else {
                "crate"
            },
            Span::call_site().into(),
        );

        let assertion = quote::format_ident!("assert_is_{name}");
        let Item::Static(ref item) = item else {
            abort!(item, "Expected a `static`");
        };
        let ty = &item.ty;
        quote::quote! {
            const _: () = #hal::__macro_implementation::#assertion::<#ty>();
        }
    });

    let output = quote::quote! {
        #section
        #item
        #trait_check
    };

    output.into()
}

#[cfg(feature = "embassy")]
#[proc_macro_attribute]
// 384
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    use self::embassy::{
        main::{main, run},
        Args,
    };

    let args = syn::parse_macro_input!(args as Args);
    let f = syn::parse_macro_input!(item as syn::ItemFn);

    run(&args.meta, f, main()).unwrap_or_else(|x| x).into()
}
