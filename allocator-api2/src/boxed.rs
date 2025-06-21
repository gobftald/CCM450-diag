/// A pointer type for heap allocation.
///
/// See the [module-level documentation](../../std/boxed/index.html) for more.
// 168
use core::mem;
use core::ops::{Deref, DerefMut};

// 171
use core::ptr::{self, NonNull};

// 174
use super::alloc::{AllocError, Allocator, Global, Layout};
use super::raw_vec::RawVec;
use super::unique::Unique;

// 180
use alloc_crate::alloc::handle_alloc_error;

// 185
pub struct Box<T: ?Sized, A: Allocator = Global>(Unique<T>, A);

// 205
impl<T> Box<T> {
    #[inline(always)]
    #[must_use]
    // 220
    pub fn new(x: T) -> Self {
        Self::new_in(x, Global)
    }
}

// 356
impl<T, A: Allocator> Box<T, A> {
    /// Allocates memory in the given allocator then places `x` into it.
    ///
    /// This doesn't actually allocate if `T` is zero-sized.
    ///
    #[must_use]
    #[inline(always)]
    // 371
    pub fn new_in(x: T, alloc: A) -> Self
    where
        A: Allocator,
    {
        let mut boxed = Self::new_uninit_in(alloc);
        unsafe {
            boxed.as_mut_ptr().write(x);
            boxed.assume_init()
        }
    }

    /// Constructs a new box with uninitialized contents in the provided allocator.
    ///
    #[inline(always)]
    // 429
    pub fn new_uninit_in(alloc: A) -> Box<mem::MaybeUninit<T>, A>
    where
        A: Allocator,
    {
        let layout = Layout::new::<mem::MaybeUninit<T>>();
        // NOTE: Prefer match over unwrap_or_else since closure sometimes not inlineable.
        // That would make code size bigger.
        match Box::try_new_uninit_in(alloc) {
            Ok(m) => m,
            Err(_) => handle_alloc_error(layout),
        }
    }

    /// Constructs a new box with uninitialized contents in the provided allocator,
    /// returning an error if the allocation fails
    ///
    #[inline(always)]
    // 463
    pub fn try_new_uninit_in(alloc: A) -> Result<Box<mem::MaybeUninit<T>, A>, AllocError>
    where
        A: Allocator,
    {
        let ptr = if mem::size_of::<T>() == 0 {
            NonNull::dangling()
        } else {
            let layout = Layout::new::<mem::MaybeUninit<T>>();
            alloc.allocate(layout)?.cast()
        };

        unsafe { Ok(Box::from_raw_in(ptr.as_ptr(), alloc)) }
    }
}

// 599
impl<T> Box<[T]> {
    /// Constructs a new boxed slice with uninitialized contents.
    ///
    #[must_use]
    #[inline(always)]
    pub fn new_uninit_slice(len: usize) -> Box<[mem::MaybeUninit<T>]> {
        unsafe { RawVec::with_capacity(len).into_box(len) }
    }
}

// 857
impl<T, A: Allocator> Box<mem::MaybeUninit<T>, A> {
    /// Converts to `Box<T, A>`.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`],
    /// it is up to the caller to guarantee that the value
    /// really is in an initialized state.
    /// Calling this when the content is not yet fully initialized
    /// causes immediate undefined behavior.
    #[inline(always)]
    // 886
    pub unsafe fn assume_init(self) -> Box<T, A> {
        let (raw, alloc) = Self::into_raw_with_allocator(self);
        unsafe { Box::<T, A>::from_raw_in(raw as *mut T, alloc) }
    }
}

// 1068
impl<T: ?Sized, A: Allocator> Box<T, A> {
    #[inline(always)]
    // 1113
    pub const unsafe fn from_raw_in(raw: *mut T, alloc: A) -> Self {
        Box(unsafe { Unique::new_unchecked(raw) }, alloc)
    }

    /// Consumes the `Box`, returning a wrapped raw pointer.
    ///
    #[inline(always)]
    pub fn into_raw(b: Self) -> *mut T {
        Self::into_raw_with_allocator(b).0
    }

    #[inline(always)]
    // 1313
    pub fn into_raw_with_allocator(b: Self) -> (*mut T, A) {
        let (leaked, alloc) = Box::into_non_null_with_allocator(b);
        (leaked.as_ptr(), alloc)
    }

    #[inline(always)]
    // 1360
    pub fn into_non_null_with_allocator(b: Self) -> (NonNull<T>, A) {
        // Box is recognized as a "unique pointer" by Stacked Borrows, but internally it is a
        // raw pointer for the type system. Turning it directly into a raw pointer would not be
        // recognized as "releasing" the unique pointer to permit aliased raw accesses,
        // so all raw pointer methods have to go through `Box::leak`. Turning *that* to a raw pointer
        // behaves correctly.
        let alloc = unsafe { ptr::read(&b.1) };
        (NonNull::from(Box::leak(b)), alloc)
    }

    /// Consumes and leaks the `Box`, returning a mutable reference,
    /// `&'a mut T`. Note that the type `T` must outlive the chosen lifetime
    /// `'a`. If the type has only static references, or none at all, then this
    /// may be chosen to be `'static`.
    ///
    /// This function is mainly useful for data that lives for the remainder of
    /// the program's life. Dropping the returned reference will cause a memory
    /// leak. If this is not acceptable, the reference should first be wrapped
    /// with the [`Box::from_raw`] function producing a `Box`. This `Box` can
    /// then be dropped which will properly destroy `T` and release the
    /// allocated memory.
    ///
    /// Note: this is an associated function, which means that you have
    /// to call it as `Box::leak(b)` instead of `b.leak()`. This
    /// is so that there is no conflict with a method on the inner type.
    ///
    #[inline(always)]
    pub fn leak<'a>(b: Self) -> &'a mut T
    where
        A: 'a,
    {
        unsafe { &mut *mem::ManuallyDrop::new(b).0.as_ptr() }
    }
}

// 2156
impl<T: ?Sized, A: Allocator> Deref for Box<T, A> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

// 2165
impl<T: ?Sized, A: Allocator> DerefMut for Box<T, A> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}
