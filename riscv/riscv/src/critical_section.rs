use critical_section::{set_impl, Impl, RawRestoreState};

use crate::interrupt;

struct SingleHartCriticalSection;
set_impl!(SingleHartCriticalSection);

unsafe impl Impl for SingleHartCriticalSection {
    #[cfg(not(feature = "s-mode"))]
    unsafe fn acquire() -> RawRestoreState {
        let mut mstatus: usize;
        core::arch::asm!("csrrci {}, mstatus, 0b1000", out(reg) mstatus);
        //core::mem::transmute::<_, crate::register::mstatus::Mstatus>(mstatus).mie()
        core::mem::transmute::<usize, crate::register::mstatus::Mstatus>(mstatus).mie()
    }

    unsafe fn release(was_active: RawRestoreState) {
        // Only re-enable interrupts if they were enabled before the critical section.
        if was_active {
            interrupt::enable()
        }
    }
}
