//! # System Control

//use crate::peripherals::SYSTEM;
use esp_sync::NonReentrantMutex;

/*
/// Peripherals which can be enabled via `PeripheralClockControl`.
///
/// This enum represents various hardware peripherals that can be enabled
/// by the system's clock control. Depending on the target device, different
/// peripherals will be available for enabling.
// FIXME: This enum needs to be public because it's exposed via a bunch of traits, but it's not
// useful to users.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
// 20
pub enum Peripheral {
    /// Timer Group 0 peripheral.
    //#[cfg(timg0)]
    #[cfg(timergroup)]
    // 77
    Timg0,

    /// UART0 peripheral.
    //#[cfg(uart0)]
    // 89
    //Uart0,

    /// UART1 peripheral.
    // 91
    //#[cfg(uart1)]
    #[cfg(soc_has_uart1)]
    Uart1,

    /// Systimer peripheral.
    #[cfg(systimer)]
    // 119
    Systimer,
}

// 125
impl Peripheral {
    const KEEP_ENABLED: &[Peripheral] = &[
        //Peripheral::Uart0,
        #[cfg(systimer)]
        // 131
        Peripheral::Systimer,
        // 132
        Peripheral::Timg0,
    ];

    // 140
    const ALL: &[Self] = &[
        //#[cfg(timg0)]
        #[cfg(timergroup)]
        // 178
        Self::Timg0,
        //#[cfg(uart0)]
        // 186
        //Self::Uart0,
        //#[cfg(uart1)]
        #[cfg(soc_has_uart1)]
        // 188
        Self::Uart1,
        #[cfg(systimer)]
        // 206
        Self::Systimer,
    ];
}
*/

// Implements the Peripheral enum based on esp-metadata/device.soc/peripheral_clocks
// 6
implement_peripheral_clocks!();

// 18
struct RefCounts {
    counts: [usize; Peripheral::COUNT],
}

// 22
impl RefCounts {
    pub const fn new() -> Self {
        Self {
            counts: [0; Peripheral::COUNT],
        }
    }
}

// 30
static PERIPHERAL_REF_COUNT: NonReentrantMutex<RefCounts> =
    NonReentrantMutex::new(RefCounts::new());

/// Disable all peripherals.
///
/// Peripherals listed in [KEEP_ENABLED] are NOT disabled.
// 37
pub(crate) fn disable_peripherals() {
    // Take the critical section up front to avoid taking it multiple times.
    /*
    PERIPHERAL_REF_COUNT.with(|_| {
        for p in Peripheral::ALL {
            if Peripheral::KEEP_ENABLED.contains(p) {
                continue;
            }
            PeripheralClockControl::enable_internal(*p, false);
        }
    })
    */
    PERIPHERAL_REF_COUNT.with(|refcounts| {
        for p in Peripheral::KEEP_ENABLED {
            refcounts.counts[*p as usize] += 1;
        }
        for p in Peripheral::ALL {
            let ref_count = refcounts.counts[*p as usize];
            if ref_count == 0 {
                PeripheralClockControl::enable_forced_with_counts(*p, false, true, refcounts);
            }
        }
    })
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 54
pub(crate) struct PeripheralGuard {
    peripheral: Peripheral,
}

// 58
impl PeripheralGuard {
    pub(crate) fn new_with(p: Peripheral, init: fn()) -> Self {
        //if !Peripheral::KEEP_ENABLED.contains(&p) && PeripheralClockControl::enable(p) {
        if PeripheralClockControl::enable(p) {
            PeripheralClockControl::reset(p);
            init();
        }

        Self { peripheral: p }
    }

    // 68
    pub(crate) fn new(p: Peripheral) -> Self {
        Self::new_with(p, || {})
    }
}

// 83
impl Drop for PeripheralGuard {
    fn drop(&mut self) {
        //if !Peripheral::KEEP_ENABLED.contains(&self.peripheral) {
        PeripheralClockControl::disable(self.peripheral);
        //}
    }
}

/// Controls the enablement of peripheral clocks.
// 3131
pub(crate) struct PeripheralClockControl;

#[cfg(not(any(esp32c6, esp32h2)))]
// 133
impl PeripheralClockControl {
    // 140
    pub(crate) fn enable(peripheral: Peripheral) -> bool {
        PERIPHERAL_REF_COUNT.with(|ref_counts| Self::enable_with_counts(peripheral, ref_counts))
    }

    // 150
    fn enable_with_counts(peripheral: Peripheral, ref_counts: &mut RefCounts) -> bool {
        Self::enable_forced_with_counts(peripheral, true, false, ref_counts)
    }

    // 162
    pub(crate) fn disable(peripheral: Peripheral) -> bool {
        PERIPHERAL_REF_COUNT.with(|ref_counts| {
            Self::enable_forced_with_counts(peripheral, false, false, ref_counts)
        })
    }

    // 168
    fn enable_forced_with_counts(
        peripheral: Peripheral,
        enable: bool,
        force: bool,
        ref_counts: &mut RefCounts,
    ) -> bool {
        let ref_count = &mut ref_counts.counts[peripheral as usize];
        if !force {
            let prev = *ref_count;
            if enable {
                *ref_count += 1;
                trace!("Enable {:?} {} -> {}", peripheral, prev, *ref_count);
                if prev > 0 {
                    return false;
                }
            } else {
                assert!(prev != 0);
                *ref_count -= 1;
                trace!("Disable {:?} {} -> {}", peripheral, prev, *ref_count);
                if prev > 1 {
                    return false;
                }
            };
        } else if !enable {
            assert!(*ref_count == 0);
        }

        if !enable {
            unsafe { Self::reset_racey(peripheral) };
        }

        debug!("Enable {:?} {}", peripheral, enable);
        unsafe { enable_internal_racey(peripheral, enable) };

        true
    }

    // 206
    /// Resets the given peripheral
    pub(crate) unsafe fn reset_racey(peripheral: Peripheral) {
        debug!("Reset {:?}", peripheral);

        unsafe {
            assert_peri_reset_racey(peripheral, true);
            assert_peri_reset_racey(peripheral, false);
        }
    }

    /// Resets the given peripheral
    pub(crate) fn reset(peripheral: Peripheral) {
        PERIPHERAL_REF_COUNT.with(|_| unsafe { Self::reset_racey(peripheral) })
    }
}
