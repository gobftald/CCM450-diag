//! # System Control

use crate::peripherals::SYSTEM;

/// Peripherals which can be enabled via `PeripheralClockControl`.
///
/// This enum represents various hardware peripherals that can be enabled
/// by the system's clock control. Depending on the target device, different
/// peripherals will be available for enabling.
// FIXME: This enum needs to be public because it's exposed via a bunch of traits, but it's not
// useful to users.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 20
pub enum Peripheral {
    /// Systimer peripheral.
    #[cfg(systimer)]
    Systimer,
}

// 125
impl Peripheral {
    const KEEP_ENABLED: &[Peripheral] = &[
        #[cfg(systimer)]
        Peripheral::Systimer,
    ];

    const ALL: &[Self] = &[
        #[cfg(systimer)]
        Self::Systimer,
    ];
}

/// Disable all peripherals.
///
/// Peripherals listed in [KEEP_ENABLED] are NOT disabled.
// 228
pub(crate) fn disable_peripherals() {
    // Take the critical section up front to avoid taking it multiple times.
    critical_section::with(|_| {
        for p in Peripheral::ALL {
            if Peripheral::KEEP_ENABLED.contains(p) {
                continue;
            }
            PeripheralClockControl::enable_internal(*p, false);
        }
    })
}

/// Controls the enablement of peripheral clocks.
// 313
pub(crate) struct PeripheralClockControl;

#[cfg(not(any(esp32c6, esp32h2)))]
// 316
impl PeripheralClockControl {
    // 317
    fn enable_internal(peripheral: Peripheral, enable: bool) {
        debug!("Enable {:?} {}", peripheral, enable);

        if !enable {
            Self::reset(peripheral);
        }

        let system = SYSTEM::regs();

        #[cfg(not(esp32))]
        let perip_clk_en0 = &system.perip_clk_en0();

        match peripheral {
            #[cfg(systimer)]
            Peripheral::Systimer => {
                perip_clk_en0.modify(|_, w| w.systimer_clk_en().bit(enable));
            }
        }
    }

    /// Resets the given peripheral
    // 494
    pub(crate) fn reset(peripheral: Peripheral) {
        debug!("Reset {:?}", peripheral);

        let system = SYSTEM::regs();

        #[cfg(not(esp32))]
        let perip_rst_en0 = system.perip_rst_en0();

        critical_section::with(|_cs| match peripheral {
            #[cfg(systimer)]
            Peripheral::Systimer => {
                perip_rst_en0.modify(|_, w| w.systimer_rst().set_bit());
                perip_rst_en0.modify(|_, w| w.systimer_rst().clear_bit());
            }
        });
    }
}

// 1090
impl PeripheralClockControl {
    /// Enables the given peripheral.
    // 1097
    pub(crate) fn enable(peripheral: Peripheral) {
        critical_section::with(|_| Self::enable_internal(peripheral, true));
    }
}
