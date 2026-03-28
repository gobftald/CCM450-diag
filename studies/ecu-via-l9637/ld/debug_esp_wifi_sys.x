.esp_wifi_sys : ALIGN(4)
{
    *(.text.coexist_printf)
    /* it also resolves
    rtc_printf
    phy_printf
    net80211_printf
    pp_printf
    */
    
} > ROTEXT