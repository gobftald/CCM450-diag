/// Convenience alias for the [Result](core::result::Result) type for the library.
pub type Result<T> = core::result::Result<T, Error>;

/// Represents error variants for the library.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Attempted out-of-bounds access.
    IndexOutOfBounds {
        index: usize,
        min: usize,
        max: usize,
    },
    /// Invalid field value.
    InvalidFieldValue {
        field: &'static str,
        value: usize,
        bitmask: usize,
    },
    /// Invalid value of a register field that does not match any known variants.
    InvalidFieldVariant { field: &'static str, value: usize },
    /// Invalid value.
    InvalidValue { value: usize, bitmask: usize },
    /// Invalid value that does not match any known variants.
    InvalidVariant(usize),
    /// Unimplemented function or type.
    Unimplemented,
}
