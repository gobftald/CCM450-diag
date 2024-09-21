/// Convenience macro to wrap the `csrrs` assembly instruction for reading a CSR register.
#[macro_export]
macro_rules! read_csr {
    ($csr_number:literal) => {
        /// Reads the CSR.
        ///
        /// **WARNING**: panics on non-`riscv` targets.
        #[inline]
        unsafe fn _read() -> usize {
            _try_read().unwrap()
        }

        /// Attempts to read the CSR.
        #[inline]
        unsafe fn _try_read() -> $crate::result::Result<usize> {
            match () {
                #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
                () => {
                    let r: usize;
                    core::arch::asm!(concat!("csrrs {0}, ", stringify!($csr_number), ", x0"), out(reg) r);
                    Ok(r)
                }

                #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
                () => Err($crate::result::Error::Unimplemented),
            }
        }
    };
}

/// Convenience macro to read a CSR register value as a `register` type.
///
/// The `register` type must be a defined type in scope of the macro call.
#[macro_export]
// 73
macro_rules! read_csr_as {
    ($register:ident, $csr_number:literal) => {
        $crate::read_csr!($csr_number);

        /// Reads the CSR.
        ///
        /// **WARNING**: panics on non-`riscv` targets.
        #[inline]
        pub fn read() -> $register {
            $register {
                bits: unsafe { _read() },
            }
        }

        /// Attempts to reads the CSR.
        #[inline]
        pub fn try_read() -> $crate::result::Result<$register> {
            Ok($register {
                bits: unsafe { _try_read()? },
            })
        }
    };
}

/// Convenience macro to read a CSR register value as a [`usize`].
#[macro_export]
macro_rules! read_csr_as_usize {
    ($csr_number:literal) => {
        $crate::read_csr!($csr_number);

        /// Reads the CSR.
        ///
        /// **WARNING**: panics on non-`riscv` targets.
        #[inline]
        pub fn read() -> usize {
            unsafe { _read() }
        }

        /// Attempts to read the CSR.
        #[inline]
        pub fn try_read() -> $crate::result::Result<usize> {
            unsafe { _try_read() }
        }
    };
}

/// Convenience macro to wrap the `csrrw` assembly instruction for writing to CSR registers.
///
/// This macro should generally not be called directly.
///
/// Instead, use the [write_csr_as_usize](crate::write_csr_as_usize) macro.
#[macro_export]
// 175
macro_rules! write_csr {
    ($csr_number:literal) => {
        /// Writes the CSR.
        ///
        /// **WARNING**: panics on non-`riscv` targets.
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _write(bits: usize) {
            _try_write(bits).unwrap();
        }

        /// Attempts to write the CSR.
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _try_write(bits: usize) -> $crate::result::Result<()> {
            match () {
                #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
                () => {
                    core::arch::asm!(concat!("csrrw x0, ", stringify!($csr_number), ", {0}"), in(reg) bits);
                    Ok(())
                }

                #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
                () => Err($crate::result::Error::Unimplemented),
            }
        }
    };
}

/// Convenience macro around the `csrrs` assembly instruction to set the CSR register.
///
/// This macro is intended for use with the [set_csr](crate::set_csr) or [set_clear_csr](crate::set_clear_csr) macros.
#[macro_export]
// 331
macro_rules! set {
    ($csr_number:literal) => {
        /// Set the CSR.
        ///
        /// **WARNING**: panics on non-`riscv` targets.
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _set(bits: usize) {
            _try_set(bits).unwrap();
        }

        /// Attempts to set the CSR.
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _try_set(bits: usize) -> $crate::result::Result<()> {
            match () {
                #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
                () => {
                    core::arch::asm!(concat!("csrrs x0, ", stringify!($csr_number), ", {0}"), in(reg) bits);
                    Ok(())
                }

                #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
                () => Err($crate::result::Error::Unimplemented),
            }
        }
    };
}

/// Convenience macro around the `csrrc` assembly instruction to clear the CSR register.
///
/// This macro is intended for use with the [clear_csr](crate::clear_csr) or [set_clear_csr](crate::set_clear_csr) macros.
#[macro_export]
// 397
macro_rules! clear {
    ($csr_number:literal) => {
        /// Clear the CSR.
        ///
        /// **WARNING**: panics on non-`riscv` targets.
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _clear(bits: usize) {
            _try_clear(bits).unwrap();
        }

        /// Attempts to clear the CSR.
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _try_clear(bits: usize) -> $crate::result::Result<()> {
            match () {
                #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
                () => {
                    core::arch::asm!(concat!("csrrc x0, ", stringify!($csr_number), ", {0}"), in(reg) bits);
                    Ok(())
                }

                #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
                () => Err($crate::result::Error::Unimplemented),
            }
        }
    };
}

/// Convenience macro to define field setter functions for a CSR type.
#[macro_export]
// 461
macro_rules! set_csr {
    ($(#[$attr:meta])*, $set_field:ident, $e:expr) => {
        $(#[$attr])*
        #[inline]
        pub unsafe fn $set_field() {
            _set($e);
        }
    };
}

/// Convenience macro to define field clear functions for a CSR type.
#[macro_export]
// 473
macro_rules! clear_csr {
    ($(#[$attr:meta])*, $clear_field:ident, $e:expr) => {
        $(#[$attr])*
        #[inline]
        pub unsafe fn $clear_field() {
            _clear($e);
        }
    };
}

/// Convenience macro to define field setter and clear functions for a CSR type.
#[macro_export]
// 485
macro_rules! set_clear_csr {
    ($(#[$attr:meta])*, $set_field:ident, $clear_field:ident, $e:expr) => {
        $crate::set_csr!($(#[$attr])*, $set_field, $e);
        $crate::clear_csr!($(#[$attr])*, $clear_field, $e);
    }
}
