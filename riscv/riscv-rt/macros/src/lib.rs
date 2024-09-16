#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro2::Span;
use syn::{
    parse, punctuated::Punctuated, spanned::Spanned, FnArg, ItemFn, PatType, ReturnType, Type,
    Visibility,
};

use proc_macro::TokenStream;

/// Attribute to declare the entry point of the program
///
/// **IMPORTANT**: This attribute must appear exactly *once* in the dependency graph.
///
/// The specified function will be called by the reset handler *after* RAM has been initialized.
/// The type of the specified function must be `[unsafe] fn() -> !` (never ending function)
///
/// # Properties
///
/// The entry point will be called by the reset handler. The program can't reference to the entry
/// point, much less invoke it.
///
/// # Examples
///
/// - Simple entry point
///
/// ``` no_run
/// # #![no_main]
/// # use riscv_rt_macros::entry;
/// #[entry]
/// fn main() -> ! {
///     loop {
///         /* .. */
///     }
/// }
/// ```
// 53
#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    #[cfg(not(feature = "u-boot"))]
    let arguments_limit = 3;
    #[cfg(feature = "u-boot")]
    let arguments_limit = 2;

    // check the function arguments
    if f.sig.inputs.len() > arguments_limit {
        return parse::Error::new(
            f.sig.inputs.last().unwrap().span(),
            "`#[entry]` function has too many arguments",
        )
        .to_compile_error()
        .into();
    }

    fn check_correct_type(argument: &PatType, ty: &str) -> Option<TokenStream> {
        let inv_type_message = format!("argument type must be {}", ty);

        if !is_correct_type(&argument.ty, ty) {
            let error = parse::Error::new(argument.ty.span(), inv_type_message);

            Some(error.to_compile_error().into())
        } else {
            None
        }
    }

    fn check_argument_type(argument: &FnArg, ty: &str) -> Option<TokenStream> {
        let argument_error = parse::Error::new(argument.span(), "invalid argument");
        let argument_error = argument_error.to_compile_error().into();

        match argument {
            FnArg::Typed(argument) => check_correct_type(argument, ty),
            FnArg::Receiver(_) => Some(argument_error),
        }
    }

    #[cfg(not(feature = "u-boot"))]
    for argument in f.sig.inputs.iter() {
        if let Some(message) = check_argument_type(argument, "usize") {
            return message;
        };
    }

    #[cfg(feature = "u-boot")]
    if let Some(argument) = f.sig.inputs.get(0) {
        if let Some(message) = check_argument_type(argument, "c_int") {
            return message;
        }
    }

    #[cfg(feature = "u-boot")]
    if let Some(argument) = f.sig.inputs.get(1) {
        if let Some(message) = check_argument_type(argument, "*const *const c_char") {
            return message;
        }
    }

    // check the function signature
    let valid_signature = f.sig.constness.is_none()
        && f.sig.asyncness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none()
        && match f.sig.output {
            ReturnType::Default => false,
            ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(_)),
        };

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[entry]` function must have signature `[unsafe] fn([arg0: usize, ...]) -> !`",
        )
        .to_compile_error()
        .into();
    }

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    // XXX should we blacklist other attributes?
    let attrs = f.attrs;
    let unsafety = f.sig.unsafety;
    let args = f.sig.inputs;
    let stmts = f.block.stmts;

    quote!(
        #[allow(non_snake_case)]
        #[export_name = "main"]
        #(#attrs)*
        pub #unsafety fn __risc_v_rt__main(#args) -> ! {
            #(#stmts)*
        }
    )
    .into()
}

// 155
fn strip_type_path(ty: &Type) -> Option<Type> {
    match ty {
        Type::Ptr(ty) => {
            let mut ty = ty.clone();
            ty.elem = Box::new(strip_type_path(&ty.elem)?);
            Some(Type::Ptr(ty))
        }
        Type::Path(ty) => {
            let mut ty = ty.clone();
            let last_segment = ty.path.segments.last().unwrap().clone();
            ty.path.segments = Punctuated::new();
            ty.path.segments.push_value(last_segment);
            Some(Type::Path(ty))
        }
        _ => None,
    }
}

// 174
fn is_correct_type(ty: &Type, name: &str) -> bool {
    let correct: Type = syn::parse_str(name).unwrap();
    if let Some(ty) = strip_type_path(ty) {
        ty == correct
    } else {
        false
    }
}
