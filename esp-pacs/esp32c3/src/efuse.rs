#[repr(C)]
//#[cfg_attr(feature = "impl-register-debug", derive(Debug))]
/// Register block
pub struct RegisterBlock {
    _reserved0: [u8; 0x44],
    rd_mac_spi_sys_0: RD_MAC_SPI_SYS_0,
}

impl RegisterBlock {
    /// 0x44 - BLOCK1 data register 0.
    #[inline(always)]
    pub const fn rd_mac_spi_sys_0(&self) -> &RD_MAC_SPI_SYS_0 {
        &self.rd_mac_spi_sys_0
    }
}

/// RD_MAC_SPI_SYS_0 (r) register accessor: BLOCK1 data register 0.
pub type RD_MAC_SPI_SYS_0 = crate::Reg<rd_mac_spi_sys_0::RD_MAC_SPI_SYS_0_SPEC>;
pub mod rd_mac_spi_sys_0;
