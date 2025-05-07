use core::marker::PhantomData;

/// A software interrupt can be triggered by software.
#[non_exhaustive]
// 57
pub struct SoftwareInterrupt<'d, const NUM: u8> {
    _lifetime: PhantomData<&'d mut ()>,
}

// 61
impl<const NUM: u8> SoftwareInterrupt<'_, NUM> {
    /// Unsafely create an instance of this peripheral out of thin air.
    ///
    /// # Safety
    ///
    /// You must ensure that you're only using one instance of this type at a
    /// time.
    #[inline]
    // 69
    pub unsafe fn steal() -> Self {
        Self {
            _lifetime: PhantomData,
        }
    }

    /// Trigger this software-interrupt
    // 102
    pub fn raise(&self) {
        let system = crate::peripherals::SYSTEM::regs();

        match NUM {
            3 => system
                .cpu_intr_from_cpu_3()
                .write(|w| w.cpu_intr_from_cpu_3().set_bit()),
            _ => unreachable!(),
        };
    }
}
