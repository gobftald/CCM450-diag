//! Build utilities for esp-hal.

use std::{io::Write as _, process};

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, punctuated::Punctuated, LitStr, Token};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[proc_macro]
// 26
pub fn error(input: TokenStream) -> TokenStream {
    do_alert(Color::Red, input);
    process::exit(1);
}

/// Given some features, assert that **exactly** one of the features is enabled.
///
/// ## Example
/// ```rust
/// assert_unique_used_features!("foo", "bar", "baz");
/// ```
#[proc_macro]
// 98
pub fn assert_unique_used_features(input: TokenStream) -> TokenStream {
    let features = parse_macro_input!(input with Punctuated<LitStr, Token![,]>::parse_terminated)
        .into_iter()
        .collect::<Vec<_>>();

    let unique = impl_unique_features(&features, "exactly one");
    let used = impl_used_features(&features, "exactly one");

    quote::quote! {
        #unique
        #used
    }
    .into()
}

// 116
fn impl_unique_features(features: &[LitStr], expectation: &str) -> impl ToTokens {
    let pairs = unique_pairs(features);
    let unique_cfgs = pairs
        .iter()
        .map(|(a, b)| quote::quote! { all(feature = #a, feature = #b) });

    let message = format!(
        r#"
ERROR: expected {expectation} enabled feature from feature group:
  {:?}
"#,
        features.iter().map(|lit| lit.value()).collect::<Vec<_>>(),
    );

    quote::quote! {
        #[cfg(any(#(#unique_cfgs),*))]
        ::esp_build::error! { #message }
    }
}

// 136
fn impl_used_features(features: &[LitStr], expectation: &str) -> impl ToTokens {
    let message = format!(
        r#"
ERROR: expected {expectation} enabled feature from feature group:
  {:?}
    "#,
        features.iter().map(|lit| lit.value()).collect::<Vec<_>>()
    );

    quote::quote! {
        #[cfg(not(any(#(feature = #features),*)))]
        ::esp_build::error! { #message }
    }
}

// 153
fn do_alert(color: Color, input: TokenStream) -> TokenStream {
    let message = parse_macro_input!(input as LitStr).value();

    let stderr = &mut StandardStream::stderr(ColorChoice::Auto);
    let color_spec = ColorSpec::new().set_fg(Some(color)).clone();

    let mut has_nonspace = false;

    for mut line in message.lines() {
        if !has_nonspace {
            let (maybe_heading, rest) = split_heading(line);

            if let Some(heading) = maybe_heading {
                stderr.set_color(color_spec.clone().set_bold(true)).ok();
                write!(stderr, "\n{}", heading).ok();
                has_nonspace = true;
            }

            line = rest;
        }

        if line.is_empty() {
            writeln!(stderr).ok();
        } else {
            stderr.set_color(&color_spec).ok();
            writeln!(stderr, "{}", line).ok();

            has_nonspace = has_nonspace || line.contains(|ch: char| ch != ' ');
        }
    }

    stderr.reset().ok();
    writeln!(stderr).ok();

    TokenStream::new()
}

// 190
fn split_heading(s: &str) -> (Option<&str>, &str) {
    let mut end = 0;
    while end < s.len() && s[end..].starts_with(|ch: char| ch.is_ascii_uppercase()) {
        end += 1;
    }

    if end >= 3 && (end == s.len() || s[end..].starts_with(':')) {
        let (heading, rest) = s.split_at(end);
        (Some(heading), rest)
    } else {
        (None, s)
    }
}

// 206
fn unique_pairs(features: &[LitStr]) -> Vec<(&LitStr, &LitStr)> {
    let mut pairs = Vec::new();

    let mut i = 0;
    let mut j = 0;

    while i < features.len() {
        let a = &features[i];
        let b = &features[j];

        if a.value() != b.value() {
            pairs.push((a, b));
        }

        j += 1;

        if j >= features.len() {
            i += 1;
            j = i;
        }
    }

    pairs
}
