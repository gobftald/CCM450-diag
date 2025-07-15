#[macro_use]
mod atomic_32_macros {
    #[macro_export]
    macro_rules! cfg_has_atomic_32 {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }
    #[macro_export]
    macro_rules! cfg_no_atomic_32 {
        ($($tt:tt)*) => {};
    }
}
