//! # Peripheral Instances
//!
//! This module creates singleton instances for each of the various peripherals,
//! and re-exports them to allow users to access and use them in their
//! applications.
//!
//! Should be noted that that the module also re-exports the [Interrupt] enum
//! from the PAC, allowing users to handle interrupts associated with these
//! peripherals.

// Note that certain are marked with `virtual` in the invocation of the
// `peripherals!` macro below. Basically, this indicates there's no physical
// peripheral (no `PSRAM`, `RADIO`, etc. peripheral in the PACs), so we're
// creating "virtual peripherals" for them.

// 11
pub(crate) use esp32c3 as pac;

// We need to export this for users to use
// 14
pub use pac::Interrupt;

// Note that certain are marked with `virtual` in the invocation of the
// `peripherals!` macro below. Basically, this indicates there's no physical
// peripheral (no `PSRAM`, `RADIO`, etc. peripheral in the PACs), so we're
// creating "virtual peripherals" for them.
// 20
crate::peripherals! {
    peripherals: [
        I2C_ANA_MST <= I2C_ANA_MST,
        INTERRUPT_CORE0 <= INTERRUPT_CORE0,
        LPWR <= RTC_CNTL,
        SYSTEM <= SYSTEM,
        SYSTIMER <= SYSTIMER,
        TIMG0 <= TIMG0,
    ],
}
