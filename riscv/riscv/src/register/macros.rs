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
