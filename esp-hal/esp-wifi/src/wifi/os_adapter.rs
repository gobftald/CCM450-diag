/// **************************************************************************
/// Name: esp_malloc
///
/// Description:
///   Allocate a block of memory
///
/// Input Parameters:
///   size - memory size
///
/// Returned Value:
///   Memory pointer
///
/// *************************************************************************
// 824
pub unsafe extern "C" fn malloc(size: usize) -> *mut crate::binary::c_types::c_void {
    unsafe { crate::compat::malloc::malloc(size).cast() }
}

/// **************************************************************************
/// Name: esp_free
///
/// Description:
///   Free a block of memory
///
/// Input Parameters:
///   ptr - memory block
///
/// Returned Value:
///   No
///
/// *************************************************************************
// 841
pub unsafe extern "C" fn free(p: *mut crate::binary::c_types::c_void) {
    unsafe {
        crate::compat::malloc::free(p.cast());
    }
}
