#![macro_use]
#![allow(unused_macros)]
#![allow(dead_code)]

#[collapse_debuginfo(yes)]
// 5
macro_rules! assert {
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
// 17
macro_rules! assert_eq {
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
// 29
macro_rules! assert_ne {
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
// 77
macro_rules! todo {
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
// 89
macro_rules! unreachable {
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
            ::core::panic!($($x)*);
            #[cfg(feature = "defmt")]
            ::defmt::panic!($($x)*);

            // we cannot use defmt specific formatting (e.g. Type hints)
        }
    };
}

#[collapse_debuginfo(yes)]
// 113
macro_rules! trace {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::trace!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            let _ = ($( & $x ),*);
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
            let _ = ($( & $x ),*);
        }
    };
}

#[collapse_debuginfo(yes)]
// 141
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::info!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            let _ = ($( & $x ),*);
        }
    };
}

#[collapse_debuginfo(yes)]
// 155
macro_rules! warn {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::warn!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            let _ = ($( & $x ),*);
        }
    };
}

#[collapse_debuginfo(yes)]
// 169
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {
        {
            #[cfg(feature = "defmt")]
            ::defmt::error!($s $(, $x)*);
            #[cfg(not(feature="defmt"))]
            let _ = ($( & $x ),*);
        }
    };
}

#[cfg(feature = "defmt")]
#[collapse_debuginfo(yes)]
// 184
macro_rules! unwrap {
    ($($x:tt)*) => {
        ::defmt::unwrap!($($x)*)
    };
}

#[cfg(not(feature = "defmt"))]
#[collapse_debuginfo(yes)]
// 192
macro_rules! unwrap {
    ($arg:expr) => {
        match $crate::fmt::Try::into_result($arg) {
            ::core::result::Result::Ok(t) => t,
            ::core::result::Result::Err(e) => {
                ::core::panic!("unwrap of `{}` failed: {:?}", ::core::stringify!($arg), e);
            }
        }
    };

    ($arg:expr, $($msg:expr),+ $(,)? ) => {
        match $crate::fmt::Try::into_result($arg) {
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

// my extension
// putting defmt runtime and basic usb print here
#[cfg(feature = "defmt")]
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

#[cfg(feature = "defmt")]
#[defmt::global_logger]

pub struct Logger;

// using cooperative scheduler (like embassy) and not or carefully
// implementing log/print in ISRs can eliminate critical sections
#[cfg(feature = "defmt")]
#[allow(static_mut_refs)]
unsafe impl defmt::Logger for Logger {
    fn acquire() {
        // Write a non-UTF8 sequence to indicate the start of a defmt frame.
        // We need this to distinguish defmt frames from other data that
        // might be written to the printer.
        print_wo_flush(&[0xFF, 0x00]);

        // safety: accessing the `static mut` is OK because
        // we ensure not using nested defmt calls
        unsafe { ENCODER.start_frame(print_wo_flush) }
    }

    unsafe fn release() {
        // safety: accessing the `static mut` is OK because
        // we ensure not using nested defmt calls
        ENCODER.end_frame(print_wo_flush);

        Self::flush();
    }

    unsafe fn flush() {
        usb_uart_tx_flush();
    }

    unsafe fn write(bytes: &[u8]) {
        // safety: accessing the `static mut` is OK because
        // we ensure not using nested defmt calls
        ENCODER.write(bytes, print_wo_flush);
    }
}

extern "C" {
    fn usb_uart_tx_one_char(char: u8);
    fn usb_uart_tx_flush();
}

#[cfg(feature = "console")]
pub fn print_wo_flush(bytes: &[u8]) {
    unsafe {
        for byte in bytes {
            usb_uart_tx_one_char(*byte);
        }
    }
}

#[cfg(feature = "console")]
pub fn print(bytes: &[u8]) {
    print_wo_flush(bytes);
    unsafe { usb_uart_tx_flush() }
}
