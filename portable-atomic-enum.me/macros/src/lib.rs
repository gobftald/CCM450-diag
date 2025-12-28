// 46
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Attribute, Ident, ItemEnum, Variant, Visibility};

// 51
fn enum_definition<'a>(
    attrs: impl IntoIterator<Item = Attribute>,
    vis: &Visibility,
    ident: &Ident,
    variants: impl IntoIterator<Item = &'a Variant>,
) -> TokenStream2 {
    let attrs = attrs.into_iter();
    let variants = variants.into_iter();

    quote! {
        #(#attrs)*
        #vis enum #ident {
            #( #variants ),*
        }
    }
}

// 68
fn atomic_enum_definition(vis: &Visibility, ident: &Ident, atomic_ident: &Ident) -> TokenStream2 {
    let atomic_ident_docs = format!(
        "A wrapper around [`{ident}`] which can be safely shared between threads.

This type uses an `AtomicUsize` to store the enum value.",
    );

    quote! {
        #[doc = #atomic_ident_docs]
        #vis struct #atomic_ident(portable_atomic_enum::atomic::AtomicUsize);
    }
}

// 81
fn enum_to_usize(ident: &Ident) -> TokenStream2 {
    let to_usize_docs = format!("Converts the [`{ident}`] variant to a `usize`.");
    quote! {
        #[doc = #to_usize_docs]
        const fn to_usize(val: #ident) -> usize {
            val as usize
        }
    }
}

// 91
fn enum_from_usize(ident: &Ident, variants: impl IntoIterator<Item = Variant>) -> TokenStream2 {
    let from_usize_docs = format!("Converts the `usize` to a [`{ident}`] variant.");

    let variants = variants
        .into_iter()
        .map(|v| v.ident)
        .map(|id| quote! { v if v == #ident::#id as usize => #ident::#id, });

    quote! {
        #[doc = #from_usize_docs]
        fn from_usize(val: usize) -> #ident {
            match val {
                #(#variants)*
                _ => panic!("Invalid enum discriminant"),
            }
        }
    }
}

// 110
fn atomic_enum_new(ident: &Ident, atomic_ident: &Ident) -> TokenStream2 {
    let atomic_ident_docs = format!("Creates a new atomic [`{ident}`].");

    quote! {
        #[doc = #atomic_ident_docs]
        pub const fn new(v: #ident) -> #atomic_ident {
            #atomic_ident(portable_atomic_enum::atomic::AtomicUsize::new(Self::to_usize(v)))
        }
    }
}

// 167
fn atomic_enum_load(ident: &Ident) -> TokenStream2 {
    quote! {
        /// Loads a value from the atomic.
        ///
        /// `load` takes an `Ordering` argument which describes the memory ordering of this operation. Possible values are `SeqCst`, `Acquire` and `Relaxed`.
        ///
        /// # Panics
        ///
        /// Panics if order is `Release` or `AcqRel`.
        pub fn load(&self, order: ::core::sync::atomic::Ordering) -> #ident {
            Self::from_usize(self.0.load(order))
        }
    }
}

// 182
fn atomic_enum_store(ident: &Ident) -> TokenStream2 {
    quote! {
        /// Stores a value into the atomic.
        ///
        /// `store` takes an `Ordering` argument which describes the memory ordering of this operation. Possible values are `SeqCst`, `Release` and `Relaxed`.
        ///
        /// # Panics
        ///
        /// Panics if order is `Acquire` or `AcqRel`.
        pub fn store(&self, val: #ident, order: ::core::sync::atomic::Ordering) {
            self.0.store(Self::to_usize(val), order)
        }
    }
}

#[proc_macro_attribute]
/// Creates an atomic wrapper around a C-style enum.
///
/// The generated type is a wrapper around `AtomicUsize` that transparently
/// converts between the stored integer and the enum type. This attribute
/// also automatically derives the `Debug`, `Copy` and `Clone` traits on
/// the enum type.
///
/// The name of the atomic type is the name of the enum type, prefixed with
/// `Atomic`.
///
// 330
pub fn atomic_enum(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input
    let ItemEnum {
        attrs,
        vis,
        ident,
        generics,
        variants,
        ..
    } = parse_macro_input!(input as ItemEnum);

    // We only support C-style enums: No generics, no fields
    if !generics.params.is_empty() {
        return syn::Error::new_spanned(generics, "Expected an enum without generics.")
            .into_compile_error()
            .into();
    }

    for variant in variants.iter() {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return syn::Error::new_spanned(&variant.fields, "Expected a variant without fields.")
                .into_compile_error()
                .into();
        }
    }

    // Define the enum
    let mut output = enum_definition(attrs, &vis, &ident, &variants);

    // Define the atomic wrapper
    let atomic_ident = parse_macro_input!(args as Option<Ident>)
        .unwrap_or_else(|| Ident::new(&format!("Atomic{}", ident), ident.span()));
    output.extend(atomic_enum_definition(&vis, &ident, &atomic_ident));

    let enum_to_usize = enum_to_usize(&ident);
    let enum_from_usize = enum_from_usize(&ident, variants);
    let atomic_enum_new = atomic_enum_new(&ident, &atomic_ident);
    let atomic_enum_load = atomic_enum_load(&ident);
    let atomic_enum_store = atomic_enum_store(&ident);

    output.extend(quote! {
        impl #atomic_ident {
            #enum_to_usize
            #enum_from_usize
            #atomic_enum_new
            #atomic_enum_load
            #atomic_enum_store
        }
    });

    output.into()
}
