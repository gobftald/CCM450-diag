// 4
use crate::alloc::Allocator;

// 6
use super::Vec;

// 8
macro_rules! __impl_slice_eq1 {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)?) => {
        impl<T, U, $($vars)*> PartialEq<$rhs> for $lhs
        where
            T: PartialEq<U>,
            $($ty: $bound)?
        {
            #[inline(always)]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }
            #[inline(always)]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    }
}

// 23
__impl_slice_eq1! { [A1: Allocator, A2: Allocator] Vec<T, A1>, Vec<U, A2> }
