//! A contiguous growable array type with heap-allocated contents, written
//! `Vec<T>`.
//!

// 66
use core::cmp::Ordering;

// 68
use core::fmt;

// 76
use core::ops;
use core::ptr;

// 78
use core::slice;

// 83
use super::{
    alloc::{Allocator, Global},
    assume,
    raw_vec::RawVec,
};

// 104
mod partial_eq;

// 372
pub struct Vec<T, A: Allocator = Global> {
    buf: RawVec<T, A>,
    len: usize,
}

// 377
////////////////////////////////////////////////////////////////////////////////
// Inherent methods
////////////////////////////////////////////////////////////////////////////////
//381
impl<T> Vec<T> {
    /// Constructs a new, empty `Vec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    #[inline(always)]
    #[must_use]
    // 395
    pub const fn new() -> Self {
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    #[inline(always)]
    #[must_use]
    // 457
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, Global)
    }
}

// 567
impl<T, A: Allocator> Vec<T, A> {
    #[inline(always)]
    // 857
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    #[inline(always)]
    // 643
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        Vec {
            buf: RawVec::with_capacity_in(capacity, alloc),
            len: 0,
        }
    }

    /// Reserves the minimum capacity for at least `additional` more elements to
    /// be inserted in the given `Vec<T>`. Unlike [`reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already
    /// sufficient.
    ///
    #[inline(always)]
    // 882
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(self.len, additional);
    }

    /// Extracts a slice containing the entire vector.
    ///
    #[inline(always)]
    // 1180
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Extracts a mutable slice of the entire vector.
    ///
    #[inline(always)]
    // 1199
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    #[inline(always)]
    // 1232
    pub fn as_ptr(&self) -> *const T {
        // We shadow the slice method of the same name to avoid going through
        // `deref`, which creates an intermediate reference.
        let ptr = self.buf.ptr();
        unsafe {
            assume(!ptr.is_null());
        }
        ptr
    }

    #[inline(always)]
    // 1270
    pub fn as_mut_ptr(&mut self) -> *mut T {
        // We shadow the slice method of the same name to avoid going through
        // `deref_mut`, which creates an intermediate reference.
        let ptr = self.buf.ptr();
        unsafe {
            assume(!ptr.is_null());
        }
        ptr
    }

    /// Returns a reference to the underlying allocator.
    #[inline(always)]
    // 1282
    pub fn allocator(&self) -> &A {
        self.buf.allocator()
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// This is a low-level operation that maintains none of the normal
    /// invariants of the type. Normally changing the length of a vector
    /// is done using one of the safe operations instead, such as
    /// [`truncate`], [`resize`], [`extend`], or [`clear`].
    ///
    #[inline(always)]
    // 1369
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.len = new_len;
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is *O*(1).
    /// If you need to preserve the element order, use [`remove`] instead.
    ///
    #[inline(always)]
    // 1402
    pub fn swap_remove(&mut self, index: usize) -> T {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!(
                "swap_remove index (is {}) should be < len (is {})",
                index, len
            );
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }
        unsafe {
            // We replace self[index] with the last element. Note that if the
            // bounds check above succeeds there must be a last element (which
            // can be self[index] itself).
            let value = ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(len - 1), base_ptr.add(index), 1);
            self.set_len(len - 1);
            value
        }
    }

    /// Appends an element if there is sufficient spare capacity, otherwise an error is returned
    /// with the element.
    ///
    /// Unlike [`push`] this method will not reallocate when there's insufficient capacity.
    /// The caller should use [`reserve`] or [`try_reserve`] to ensure that there is enough capacity.
    ///
    #[inline(always)]
    // 1912
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        if self.len == self.buf.capacity() {
            return Err(value);
        }
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            ptr::write(end, value);
            self.len += 1;
        }
        Ok(())
    }

    /// Clears the vector, removing all values.
    ///
    #[inline(always)]
    // 2086
    pub fn clear(&mut self) {
        let elems: *mut [T] = self.as_mut_slice();

        // SAFETY:
        // - `elems` comes directly from `as_mut_slice` and is therefore valid.
        // - Setting `self.len` before calling `drop_in_place` means that,
        //   if an element's `Drop` impl panics, the vector's `Drop` impl will
        //   do nothing (leaking the rest of the elements) instead of dropping
        //   some twice.
        unsafe {
            self.len = 0;
            ptr::drop_in_place(elems);
        }
    }
}

// 2398
impl<T: Clone, A: Allocator> Vec<T, A> {
    #[inline(always)]
    // 2457
    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.extend(other.iter().cloned())
    }
}

// 2651
////////////////////////////////////////////////////////////////////////////////
// Common trait implementations for Vec
////////////////////////////////////////////////////////////////////////////////
// 2655
impl<T, A: Allocator> ops::Deref for Vec<T, A> {
    type Target = [T];

    #[inline(always)]
    // 2659
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

// 2664
impl<T, A: Allocator> ops::DerefMut for Vec<T, A> {
    #[inline(always)]
    // 2666
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }
}

// 2672
impl<T: Clone, A: Allocator + Clone> Clone for Vec<T, A> {
    #[inline(always)]
    fn clone(&self) -> Self {
        let alloc = self.allocator().clone();
        let mut vec = Vec::with_capacity_in(self.len(), alloc);
        vec.extend_from_slice(self);
        vec
    }
}

// 2788
impl<'a, T, A: Allocator> IntoIterator for &'a Vec<T, A> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Implements comparison of vectors, [lexicographically](core::cmp::Ord#lexicographical-comparison).
// 2916
impl<T: PartialOrd, A: Allocator> PartialOrd for Vec<T, A> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

// 2923
impl<T: Eq, A: Allocator> Eq for Vec<T, A> {}

// 2956
impl<T: fmt::Debug, A: Allocator> fmt::Debug for Vec<T, A> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

// 2808
impl<T, A: Allocator> Extend<T> for Vec<T, A> {
    #[inline(always)]
    // 2810
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // This is the case for a general iter.
        //
        // This function should be the moral equivalent of:
        //
        //      for item in iter {
        //          self.push(item);
        //      }

        let mut iter = iter.into_iter();
        while let Some(element) = iter.next() {
            let len = self.len();
            if len == self.capacity() {
                let (lower, _) = iter.size_hint();
                self.reserve(lower.saturating_add(1));
            }
            unsafe {
                ptr::write(self.as_mut_ptr().add(len), element);
                // Since next() executes user code which can panic we have to bump the length
                // after each step.
                // NB can't overflow since we would have had to alloc the address space
                self.set_len(len + 1);
            }
        }
    }
}

// 2946
impl<T> Default for Vec<T> {
    /// Creates an empty `Vec<T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    #[inline(always)]
    fn default() -> Vec<T> {
        Vec::new()
    }
}

#[cfg(feature = "defmt")]
impl<T, A: Allocator> defmt::Format for Vec<T, A>
where
    T: defmt::Format,
{
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(fmt, "{=[?]}", self.as_slice())
    }
}
