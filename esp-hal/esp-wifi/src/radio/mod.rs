#[cfg_attr(esp32c3, path = "radio_esp32c3.rs")]
mod chip_specific;

pub(crate) use chip_specific::*;
