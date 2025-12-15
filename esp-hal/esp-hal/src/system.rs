//! # System Control

use crate::peripherals::SYSTEM;

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

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 2424
pub(crate) struct PeripheralGuard {
    peripheral: Peripheral,
}

// 246
impl PeripheralGuard {
    pub(crate) fn new_with(p: Peripheral, init: fn()) -> Self {
        if !Peripheral::KEEP_ENABLED.contains(&p) && PeripheralClockControl::enable(p) {
            PeripheralClockControl::reset(p);
            init();
        }

        Self { peripheral: p }
    }

    pub(crate) fn new(p: Peripheral) -> Self {
        Self::new_with(p, || {})
    }
}

// 261
impl Drop for PeripheralGuard {
    fn drop(&mut self) {
        if !Peripheral::KEEP_ENABLED.contains(&self.peripheral) {
            PeripheralClockControl::disable(self.peripheral);
        }
    }
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
            //#[cfg(timg0)]
            #[cfg(timergroup)]
            // 419
            Peripheral::Timg0 => {
                #[cfg(any(esp32c3, esp32s2, esp32s3))]
                perip_clk_en0.modify(|_, w| w.timers_clk_en().bit(enable));
                perip_clk_en0.modify(|_, w| w.timergroup_clk_en().bit(enable));
            }

            /*
            #[cfg(uart0)]
            // 444
            Peripheral::Uart0 => {
                perip_clk_en0.modify(|_, w| w.uart_clk_en().bit(enable));
            }
            */
            //#[cfg(uart1)]
            #[cfg(soc_has_uart1)]
            // 448
            Peripheral::Uart1 => {
                perip_clk_en0.modify(|_, w| w.uart1_clk_en().bit(enable));
            }

            #[cfg(systimer)]
            // 483
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
            //#[cfg(timg0)]
            #[cfg(timergroup)]
            // 618
            Peripheral::Timg0 => {
                /* reset is not called for Timg0
                #[cfg(any(esp32c3, esp32s2, esp32s3))]
                perip_rst_en0.modify(|_, w| w.timers_rst().set_bit());
                perip_rst_en0.modify(|_, w| w.timergroup_rst().set_bit());
                #[cfg(any(esp32c3, esp32s2, esp32s3))]
                perip_rst_en0.modify(|_, w| w.timers_rst().clear_bit());
                perip_rst_en0.modify(|_, w| w.timergroup_rst().clear_bit());
                */
            }

            /*
            #[cfg(uart0)]
            // 653
            Peripheral::Uart0 => {
                perip_rst_en0.modify(|_, w| w.uart_rst().set_bit());
                perip_rst_en0.modify(|_, w| w.uart_rst().clear_bit());
            */
            //#[cfg(uart1)]
            #[cfg(soc_has_uart1)]
            // 658
            Peripheral::Uart1 => {
                perip_rst_en0.modify(|_, w| w.uart1_rst().set_bit());
                perip_rst_en0.modify(|_, w| w.uart1_rst().clear_bit());
            }

            #[cfg(systimer)]
            // 698
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
    pub(crate) fn enable(peripheral: Peripheral) -> bool {
        critical_section::with(|_| Self::enable_internal(peripheral, true));
        // we don't use 'force' and PERIPHERAL_REF_COUNT
        // so we always actually enable the peripheral
        true
    }

    /// Disables the given peripheral.
    ///
    // 1119
    pub(crate) fn disable(peripheral: Peripheral) -> bool {
        critical_section::with(|_| Self::enable_internal(peripheral, false));
        // we don't use 'force' and PERIPHERAL_REF_COUNT
        // so we always actually disable the peripheral
        true
    }
}
