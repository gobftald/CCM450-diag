#![allow(non_camel_case_types)]
#![no_std]
use core::marker::PhantomData;
use core::ops::Deref;

#[allow(unused_imports)]
// 11
use generic::*;
///cCommon register and bit access and modify traits
// 13
pub mod generic;

#[doc(hidden)]
// 208
pub mod interrupt;
pub use self::interrupt::Interrupt;
/// Interrupt Controller (Core 0)
// 855
pub struct INTERRUPT_CORE0 {
    _marker: PhantomData<*const ()>,
}

// 858
unsafe impl Send for INTERRUPT_CORE0 {}

// 859
impl INTERRUPT_CORE0 {
    /// Pointer to the register block
    pub const PTR: *const interrupt_core0::RegisterBlock = 0x600c_2000 as *const _;
}

// 886
impl Deref for INTERRUPT_CORE0 {
    type Target = interrupt_core0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}

/// Interrupt Controller (Core 0)
// 898
pub mod interrupt_core0;
