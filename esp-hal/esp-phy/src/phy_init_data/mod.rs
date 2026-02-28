#[cfg_attr(esp32c3, path = "phy_init_data_esp32c3.rs")]
mod chip_phy_init_data;
pub(crate) use chip_phy_init_data::PHY_INIT_DATA_DEFAULT;