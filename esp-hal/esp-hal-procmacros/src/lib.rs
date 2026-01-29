// 46
use proc_macro::TokenStream;

// 48
mod alert;
mod blocking;

// 52
mod interrupt;

// 60
mod ram;
mod rtos_main;

// 114
#[proc_macro_attribute]
pub fn ram(args: TokenStream, input: TokenStream) -> TokenStream {
    ram::ram(args.into(), input.into()).into()
}

/// Mark a function as an interrupt handler.
///
/// Optionally a priority can be specified, e.g. `#[handler(priority =
/// esp_hal::interrupt::Priority::Priority2)]`.
///
/// If no priority is given, `Priority::min()` is assumed
// 170
#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    interrupt::handler(args.into(), input.into()).into()
}

/// Creates a new instance of `esp_rtos::embassy::Executor` and declares an application entry point
/// spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it
///   can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ```rust,ignore
/// #[esp_rtos::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
// 216
#[proc_macro_attribute]
pub fn rtos_main(args: TokenStream, item: TokenStream) -> TokenStream {
    rtos_main::main(args.into(), item.into()).into()
}

// 244
#[proc_macro_attribute]
pub fn blocking_main(args: TokenStream, input: TokenStream) -> TokenStream {
    blocking::main(args, input)
}

/// Print a build error and terminate the process.
///
/// It should be noted that the error will be printed BEFORE the main function
/// is called, and as such this should NOT be thought analogous to `println!` or
/// similar utilities.
///
/// ## Example
///
/// ```rust, ignore
/// esp_hal_procmacros::error! {"
/// ERROR: something really bad has happened!
/// "}
/// // Process exits with exit code 1
/// ```
// 301
#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
    alert::do_alert(termcolor::Color::Red, input);
    panic!("Build failed");
}

/// Print a build warning.
///
/// It should be noted that the warning will be printed BEFORE the main function
/// is called, and as such this should NOT be thought analogous to `println!` or
/// similar utilities.
///
/// ## Example
///
/// ```rust,no_run
/// esp_hal_procmacros::warning! {"
/// WARNING: something unpleasant has happened!
/// "};
/// ```
// 320
#[proc_macro]
pub fn warning(input: TokenStream) -> TokenStream {
    alert::do_alert(termcolor::Color::Yellow, input)
}

// 325
macro_rules! unwrap_or_compile_error {
    ($($x:tt)*) => {
        match $($x)* {
            Ok(x) => x,
            Err(e) => {
                return e.into_compile_error()
            }
        }
    };
}

// 336
pub(crate) use unwrap_or_compile_error;
