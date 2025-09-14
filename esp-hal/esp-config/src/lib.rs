// 5
#![cfg_attr(not(feature = "build"), no_std)]

#[cfg(feature = "build")]
// 9
mod generate;

#[cfg(feature = "build")]
// 11
pub use generate::{ConfigOption, generate_config, value::Value};

/// Parse the value of an environment variable as a [bool] at compile time.
#[macro_export]
// 18
macro_rules! esp_config_bool {
    ( $var:expr ) => {
        match env!($var).as_bytes() {
            b"true" => true,
            b"false" => false,
            _ => ::core::panic!("boolean value must be either 'true' or 'false'"),
        }
    };
}

/// Parse the value of an environment variable as an integer at compile time.
#[macro_export]
// 35
macro_rules! esp_config_int {
    ( $ty:ty, $var:expr ) => {
        const { $crate::esp_config_int_parse!($ty, env!($var)) }
    };
}

/// Get the string value of an environment variable at compile time.
#[macro_export]
// 38
macro_rules! esp_config_str {
    ( $var:expr ) => {
        env!($var)
    };
}

/// Parse a string like "777" into an integer, which _can_ be used in a `const`
/// context
///
/// Not inlined into `esp_config_int` to make this easy to test.
#[macro_export]
// 55
macro_rules! esp_config_int_parse {
    ( $ty:ty, $s:expr ) => {{
        let val: $ty = match <$ty>::from_str_radix($s, 10) {
            Ok(val) => val as $ty,
            Err(_) => {
                core::assert!(false, "Unable to parse a config value as a number.");
                0
            }
        };
        val
    }};
}
