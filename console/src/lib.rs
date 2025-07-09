#![no_std]
#![macro_use]
#![allow(unused_macros)]

#[cfg(all(feature = "defmt", not(feature = "usb_uart")))]
compile_error!("defmt needs usb_uart as console");

#[cfg(feature = "defmt")]
mod defmt;

#[cfg(feature = "usb_uart")]
mod usb_uart;
#[cfg(feature = "usb_uart")]
pub use usb_uart::{Printer, print};

#[collapse_debuginfo(yes)]
#[macro_export]
// 5
macro_rules! assert {                               // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::assert!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::assert!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 17
macro_rules! assert_eq {                            // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::assert_eq!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::assert_eq!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 29
macro_rules! assert_ne {                            // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::assert_ne!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::assert_ne!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 5
macro_rules! debug_assert {                               // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::debug_assert!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug_assert!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 17
macro_rules! debug_assert_eq {                            // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::debug_assert_eq!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug_assert_eq!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 29
macro_rules! debug_assert_ne {                            // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::debug_assert_ne!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::debug_assert_ne!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 77
macro_rules! todo {                                 // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::todo!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::todo!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 89
macro_rules! unreachable {                          // behaves as panic, see comments there
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::unreachable!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::unreachable!($($x)*);
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 101
macro_rules! panic {
    ($($x:tt)*) => {
        {
            #[cfg(not(feature = "defmt"))]
            ::core::panic!($($x)*);             // if build-std-features = ["panic_immediate_abort"]
                                                // message str does not matter, since panic will not
                                                // call panic_fmt,
                                                // so neither the massage str nor the file and line
                                                // str are stored (and occupy space) in the code
            #[cfg(feature = "defmt")]
            ::defmt::panic!($($x)*);

            // we cannot use defmt specific formatting (e.g. Type hints)
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 113
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::trace!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            { /* let _ = ($( & $x ),*); */ }
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 127
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::debug!($s $(, $x)*);
            #[cfg(not(feature = "defmt"))]
            { /* let _ = ($( & $x ),*); */ }
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 141
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::info!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            { /* let _ = ($( & $x ),*); */ }
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 155
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::warn!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            { /* let _ = ($( & $x ),*); */ }
        }
    };
}

#[collapse_debuginfo(yes)]
#[macro_export]
// 169
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::error!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            { /* let _ = ($( & $x ),*); */ }
        }
    };
}

#[cfg(feature = "defmt")]
#[collapse_debuginfo(yes)]
#[macro_export]
// 184
macro_rules! unwrap {
    ($($x:tt)*) => {
        ::defmt::unwrap!($($x)*)
    };
}

#[cfg(not(feature = "defmt"))]
#[collapse_debuginfo(yes)]
#[macro_export]
// 192
macro_rules! unwrap {
    ($arg:expr) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {:?}", ::core::stringify!($arg), e);
            }
        }
    };

    ($arg:expr, $($msg:expr),+ $(,)? ) => {
        match $crate::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {}: {:?}", ::core::stringify!($arg), ::core::format_args!($($msg,)*), e);
            }
        }
    }
}

// 221
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NoneError;

// 214
pub trait Try {
    type Ok;
    type Error;
    #[allow(unused)]
    fn into_result(self) -> Result<Self::Ok, Self::Error>;
}

// 221
impl<T> Try for Option<T> {
    type Ok = T;
    type Error = NoneError;

    #[inline]
    fn into_result(self) -> Result<T, NoneError> {
        self.ok_or(NoneError)
    }
}

impl<T, E> Try for Result<T, E> {
    type Ok = T;
    type Error = E;

    #[inline]
    fn into_result(self) -> Self {
        self
    }
}

#[cfg(feature = "usb_uart")]
#[collapse_debuginfo(yes)]
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        #[cfg(feature = "defmt")]
        ::defmt::println!($($arg)*);
        #[cfg(not(feature="defmt"))]
        {}
    };
}

#[cfg(feature = "usb_uart")]
#[collapse_debuginfo(yes)]
#[macro_export]
// when using always use:
// #[cfg(not(feature = "defmt"))]
// use core::fmt::Write;
macro_rules! core_println {
    ($($arg:tt)*) => {
        write!($crate::Printer, $($arg)*).ok();
        $crate::print(b"\n");
    }
}
