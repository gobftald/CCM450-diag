use core::fmt;
use core::ops::{Deref, DerefMut};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

/// A managed slice.
///
/// This enum can be used to represent exclusive access to slices of objects.
/// In Rust, exclusive access to an object is obtained by either owning the object,
/// or owning a mutable pointer to the object; hence, "managed".
///
/// The purpose of this enum is providing good ergonomics with `std` present while making
/// it possible to avoid having a heap at all (which of course means that `std` is not present).
/// To achieve this, the variants other than `Borrowed` are only available when the corresponding
/// feature is opted in.
///
/// A function that requires a managed object should be generic over an `Into<ManagedSlice<'a, T>>`
/// argument; then, it will be possible to pass either a `Vec<T>`, or a `&'a mut [T]`
/// without any conversion at the call site.
///
// 29
pub enum ManagedSlice<'a, T: 'a> {
    /// Borrowed variant.
    Borrowed(&'a mut [T]),
    /// Owned variant, only available with the `std` or `alloc` feature enabled.
    #[cfg(any(feature = "std", feature = "alloc"))]
    Owned(Vec<T>),
}

// 37
impl<'a, T: 'a> fmt::Debug for ManagedSlice<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ManagedSlice::Borrowed(x) => write!(f, "Borrowed({:?})", x),
            #[cfg(any(feature = "std", feature = "alloc"))]
            ManagedSlice::Owned(x) => write!(f, "Owned({:?})", x),
        }
    }
}

// 48
impl<'a, T: 'a> From<&'a mut [T]> for ManagedSlice<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        ManagedSlice::Borrowed(value)
    }
}

// 73
#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, T: 'a> From<Vec<T>> for ManagedSlice<'a, T> {
    fn from(value: Vec<T>) -> Self {
        ManagedSlice::Owned(value)
    }
}

// 80
impl<'a, T: 'a> Deref for ManagedSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            ManagedSlice::Borrowed(value) => value,
            #[cfg(any(feature = "std", feature = "alloc"))]
            ManagedSlice::Owned(value) => value,
        }
    }
}

// 92
impl<'a, T: 'a> DerefMut for ManagedSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            &mut ManagedSlice::Borrowed(ref mut value) => value,
            #[cfg(any(feature = "std", feature = "alloc"))]
            &mut ManagedSlice::Owned(ref mut value) => value,
        }
    }
}
