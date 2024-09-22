// 3
mod single_core {
    // 4
    pub unsafe fn disable_interrupts() -> critical_section::RawRestoreState {
        #[cfg(riscv)]
        {
            let mut mstatus = 0u32;
            // MIE bit -  Global machine mode interrupt enable.
            core::arch::asm!("csrrci {0}, mstatus, 8", inout(reg) mstatus);
            ((mstatus & 0b1000) != 0) as critical_section::RawRestoreState
        }
    }

    // 20
    pub unsafe fn reenable_interrupts(token: critical_section::RawRestoreState) {
        #[cfg(riscv)]
        {
            if token != 0 {
                esp_riscv_rt::riscv::interrupt::enable();
            }
        }
    }
}

// 109
pub(crate) struct Lock {
    #[cfg(multi_core)]
    inner: multicore::AtomicLock,
}

// 114
impl Lock {
    // 115
    pub const fn new() -> Self {
        Self {
            #[cfg(multi_core)]
            inner: multicore::AtomicLock::new(),
        }
    }

    // 122
    fn acquire(&self) -> critical_section::RawRestoreState {
        #[cfg(single_core)]
        unsafe {
            single_core::disable_interrupts()
        }
    }

    /// # Safety
    /// This function must only be called if the lock was acquired by the
    /// current thread.
    // 164
    unsafe fn release(&self, token: critical_section::RawRestoreState) {
        single_core::reenable_interrupts(token);
    }
}

// 240
struct CriticalSection;

// 242
critical_section::set_impl!(CriticalSection);

// 244
static CRITICAL_SECTION: Lock = Lock::new();

// 246
unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        CRITICAL_SECTION.acquire()
    }

    unsafe fn release(token: critical_section::RawRestoreState) {
        CRITICAL_SECTION.release(token);
    }
}
