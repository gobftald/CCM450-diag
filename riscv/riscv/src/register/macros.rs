/// Convenience macro to wrap the `csrrs` assembly instruction for reading a CSR register.
///
/// This macro should generally not be called directly.
///
/// Instead, use the [read_csr_as](crate::read_csr_as) or [read_csr_as_usize](crate::read_csr_as_usize) macros.
#[macro_export]
// 7
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
