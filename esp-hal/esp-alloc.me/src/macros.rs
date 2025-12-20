#[macro_export]
// 14
macro_rules! heap_allocator {
    ($(#[$m:meta])* size: $size:expr) => {{
        $(#[$m])*
        // HEAP(_MEM)
        static mut HEAP: core::mem::MaybeUninit<[u8; $size]> = core::mem::MaybeUninit::uninit();

        unsafe {
            // HEAP(_CTRL)
            //$crate::HEAP.add_region($crate::HeapRegion::new(
            $crate::HEAP.init(
                // HEAP(_MEM)
                HEAP.as_mut_ptr() as *mut u8,
                $size,
                //$crate::MemoryCapability::Internal.into(),
            );
        }
    }};
}
