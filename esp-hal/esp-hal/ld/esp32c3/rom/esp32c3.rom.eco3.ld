/*
ESP32C3 ECO3 ROM address table
Version 3 API's imported from the ROM
*/

/* 6 */
/*esf_buf_alloc_dynamic = 0x400015c0;*/
/* 7 */
/*esf_buf_recycle = 0x400015c4;*/
/* 8 */
/*lmacTxDone = 0x4000162c;*/
/* 9 */
/*ppMapTxQueue = 0x400016d8;*/
/* 10 */
/*rcGetSched = 0x40001764;*/
/* 11 */
wDevCheckBlockError = 0x400017b4;
/* 12 */
/*ppProcTxDone = 0x40001804;*/
/* 13 */
/*sta_input = rom_sta_input;*/

/* 15 */
/***************************************
 Group rom_phy
 ***************************************/

/* 19 */
/* Functions */
/* 22 */
/* rom_set_tx_dig_gain = 0x400019f0; commented out, because it does not decrease .rwtext.wifi size */
/* 23 */
/* rom_set_txcap_reg = 0x400019f4; commented out, because it does not decrease .rwtext.wifi size */
/* 25 */
/* rom_agc_reg_init = 0x40001a54; commented out, because it does not decrease .rwtext.wifi size */
/* 26 */
/* rom_bb_reg_init = 0x40001a58; commented out, because it does not decrease .rwtext.wifi size */
/* 28 */
/* rom_phy_xpd_rf = 0x40001a78; commented out, because it does not decrease .rwtext.wifi size */
/* 31 */
/* rom_open_i2c_xpd = 0x40001af8; commented out, because it does not decrease .rwtext.wifi size */

/* 37 */
/***************************************
   Group eco3_wifi
 ***************************************/

/* 41 */
/* Functions */
/* 42 */
wdev_is_data_in_rxlist = 0x40001b2c;
/* 43 */
ppProcTxCallback = 0x40001b30;

/* 79 */
/***************************************
   Group eco3_phy
 ***************************************/

/* 83 */
/* Functions */
/* 86 */
/* rom_bt_tx_dig_gain = 0x40001bb4;  commented out, because it does not decrease .rwtext.wifi size */
/* 93 */
/* rom_wifi_agc_sat_gain = 0x40001bd0; commented out, because it does not decrease .rwtext.wifi size */
/* 95 */
/* rom_bt_filter_reg = 0x40001bd8;  commented out, because it does not decrease .rwtext.wifi size */
/* 96 */
/* rom_phy_bbpll_cal = 0x40001bdc;  commented out, because it does not decrease .rwtext.wifi size */
/* 99 */
/* rom_phy_reg_init = 0x40001be8; commented out, because it does not decrease .rwtext.wifi size */
/* 100 */
/* rom_set_chan_reg = 0x40001bec; commented out, because it does not decrease .rwtext.wifi size */
/* 113 */
/* rom_phy_freq_mem_backup = 0x40001c20; commented out, because it does not decrease .rwtext.wifi size */
/* 114 */
/* rom_phy_ant_init = 0x40001c24; commented out, because it does not decrease .rwtext.wifi size */

