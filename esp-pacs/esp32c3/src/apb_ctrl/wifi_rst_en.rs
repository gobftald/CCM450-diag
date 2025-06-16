/// APB_CTRL_WIFI_RST_EN_REG
pub struct WIFI_RST_EN_SPEC;
impl crate::RegisterSpec for WIFI_RST_EN_SPEC {
    type Ux = u32;
}

impl crate::Readable for WIFI_RST_EN_SPEC {}

impl crate::Writable for WIFI_RST_EN_SPEC {
    type Safety = crate::Unsafe;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0;
}
