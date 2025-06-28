#[unsafe(no_mangle)]
// 2
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    //trace!("alloc {}", size);

    unsafe extern "C" {
        fn esp_wifi_allocate_from_internal_ram(size: usize) -> *mut u8;
    }

    let ptr = unsafe { esp_wifi_allocate_from_internal_ram(size) };

    if ptr.is_null() {
        warn!("Unable to allocate {} bytes", size);
    }

    trace!("alloc {} {:?}", size, ptr);
    ptr
}

#[unsafe(no_mangle)]
// 19
pub unsafe extern "C" fn free(ptr: *mut u8) {
    trace!("free {:?}", ptr);

    if ptr.is_null() {
        warn!("Attempt to free null pointer");
        return;
    }

    unsafe extern "C" {
        fn esp_wifi_deallocate_internal_ram(ptr: *mut u8);
    }

    unsafe {
        esp_wifi_deallocate_internal_ram(ptr);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(number: u32, size: usize) -> *mut u8 {
    trace!("calloc {} {}", number, size);

    let total_size = number as usize * size;
    unsafe {
        let ptr = malloc(total_size);

        if !ptr.is_null() {
            for i in 0..total_size as isize {
                ptr.offset(i).write_volatile(0);
            }
        }

        ptr
    }
}

// 86
#[cfg(feature = "esp-alloc")]
#[unsafe(no_mangle)]
pub extern "C" fn esp_wifi_allocate_from_internal_ram(size: usize) -> *mut u8 {
    use core::alloc::GlobalAlloc;

    let total_size = size + 4;
    unsafe {
        //let ptr = esp_alloc::HEAP.alloc_caps(
        let ptr = esp_alloc::HEAP.alloc(
            //esp_alloc::MemoryCapability::Internal.into(),
            core::alloc::Layout::from_size_align_unchecked(total_size, 4),
        );

        if ptr.is_null() {
            return ptr;
        }

        *(ptr as *mut usize) = total_size;
        ptr.offset(4)
    }
}

#[cfg(feature = "esp-alloc")]
#[unsafe(no_mangle)]
// 106
pub extern "C" fn esp_wifi_deallocate_internal_ram(ptr: *mut u8) {
    use core::alloc::GlobalAlloc;

    unsafe {
        let ptr = ptr.offset(-4);
        let total_size = *(ptr as *const usize);

        esp_alloc::HEAP.dealloc(
            ptr,
            core::alloc::Layout::from_size_align_unchecked(total_size, 4),
        )
    }
}
