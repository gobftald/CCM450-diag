#[macro_export]
// 14
macro_rules! heap_allocator {
    ($(#[$m:meta])* size: $size:expr) => {{
        $(#[$m])*
        static mut HEAP: core::mem::MaybeUninit<[u8; $size]> = core::mem::MaybeUninit::uninit();

        unsafe {
            $crate::HEAP.add_region($crate::HeapRegion::new(
                HEAP.as_mut_ptr() as *mut u8,
                $size,
                $crate::MemoryCapability::Internal.into(),
            ));
        }
    }};
}
