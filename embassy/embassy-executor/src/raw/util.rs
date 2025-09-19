// 1
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr;

// 5
pub(crate) struct UninitCell<T>(MaybeUninit<UnsafeCell<T>>);
// 6
impl<T> UninitCell<T> {
    // 7
    pub const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    // 11
    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        unsafe { (*self.0.as_ptr()).get() }
    }

    // 16
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.as_mut_ptr() }
    }

    // 21
    #[inline(never)]
    pub unsafe fn write_in_place(&self, func: impl FnOnce() -> T) {
        // closure is evaluated here
        // since the reusult of clouser is an 'async fn' and we did not want to copy
        // that 'async fn' which is not a simple fn (function ptr) as a param in the
        // previous function calls (started at 'spawn' and leading here)
        unsafe { ptr::write(self.as_mut_ptr(), func()) }
    }

    // 25
    pub unsafe fn drop_in_place(&self) {
        unsafe { ptr::drop_in_place(self.as_mut_ptr()) }
    }
}
