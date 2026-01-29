#![no_std]

// This mod MUST go first, so that the others see its macros.
// 11
pub(crate) mod fmt;

//#[macro_use(panic, unwrap)]
//extern crate console;

//12
pub use embassy_executor_macros::task;

// 53
pub mod raw;

// 55
mod spawner;
pub use spawner::*;

/// Implementation details for embassy macros.
/// Do not use. Used for macros and HALs only. Not covered by semver guarantees.
// 238
#[cfg(feature = "nightly")]
pub mod _export {
    #[diagnostic::on_unimplemented(
        message = "task futures must resolve to `()` or `!`",
        note = "use `async fn` or change the return type to `impl Future<Output = ()>`"
    )]
    pub trait TaskReturnValue {}
    impl TaskReturnValue for () {}
    impl TaskReturnValue for Never {}

    #[allow(dead_code)]
    pub trait HasOutput {
        type Output;
    }

    impl<O> HasOutput for fn() -> O {
        type Output = O;
    }

    #[allow(dead_code)]
    pub type Never = <fn() -> ! as HasOutput>::Output;
}
