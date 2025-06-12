#[macro_export]
// 14
macro_rules! heap_allocator {
    ($(#[$m:meta])* size: $size:expr) => {{
        $(#[$m])*
        static mut HEAP: core::mem::MaybeUninit<[u8; $size]> = core::mem::MaybeUninit::uninit();
    }};
}
