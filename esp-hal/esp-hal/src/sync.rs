#[cfg(impl_critical_section)]
#[cfg(feature = "rt")]
mod critical_section {
    struct CriticalSection;

    critical_section::set_impl!(CriticalSection);

    static CRITICAL_SECTION: esp_sync::RawMutex = esp_sync::RawMutex::new();

    unsafe impl critical_section::Impl for CriticalSection {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            unsafe { CRITICAL_SECTION.acquire().inner() }
        }

        unsafe fn release(token: critical_section::RawRestoreState) {
            unsafe {
                CRITICAL_SECTION.release(esp_sync::RestoreState::new(token));
            }
        }
    }
}
