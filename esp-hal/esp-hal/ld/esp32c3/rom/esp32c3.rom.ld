/* 11 */
/***************************************
 Group common
 ***************************************/

/* Functions */
/* 30 */
PROVIDE( ets_delay_us = 0x40000050 );
/* 33 */
ets_backup_dma_copy = 0x4000005c;


/* 347 */
/***************************************
 Group clock
 ***************************************/

/* Functions */
/* 354 */
ets_update_cpu_frequency = 0x40000588;


/* 404 */
/***************************************
 Group crypto
 ***************************************/

/* Functions */
/* 415 */
crc32_le = 0x40000628;


/* 511 */
/***************************************
 Group usb_uart
 ***************************************/

 /* Functions */
 /* 518 */
 PROVIDE( usb_uart_tx_flush = 0x400008d4 );
 /* 519 */
 PROVIDE( usb_uart_tx_one_char = 0x400008d8 );


/* 1498 */
 /***************************************
 Group rom_pp
 ***************************************/

/* Functions */
/* 1503 */
esp_pp_rom_version_get = 0x400015b0;
/* 1652 */
our_instances_ptr = 0x3ff1ee44;
/* 1653 */
pTxRx = 0x3fcdf968;
/* 1654 */
lmacConfMib_ptr = 0x3fcdf964;
/* 1655 */
our_wait_eb = 0x3fcdf960;
/* 1656 */
our_tx_eb = 0x3fcdf95c;
/* 1657 */
pp_wdev_funcs = 0x3fcdf958;
/* 1658 */
g_osi_funcs_p = 0x3fcdf954;
/* 1659 */
wDevCtrl_ptr = 0x3fcdf950;
/* 1660 */
g_wdev_last_desc_reset_ptr = 0x3ff1ee40;
/* 1661 */
wDevMacSleep_ptr = 0x3fcdf94c;
/* 1662 */
g_lmac_cnt_ptr = 0x3fcdf948;
/* 1663 */
our_controls_ptr = 0x3ff1ee3c;
/* 1664 */
pp_sig_cnt_ptr = 0x3fcdf944;
/* 1665 */
g_eb_list_desc_ptr = 0x3fcdf940;
/* 1666 */
s_fragment_ptr = 0x3fcdf93c;
/* 1667 */
if_ctrl_ptr = 0x3fcdf938;
/* 1668 */
g_intr_lock_mux = 0x3fcdf934;
/* 1669 */
g_wifi_global_lock = 0x3fcdf930;
/* 1670 */
s_wifi_queue = 0x3fcdf92c;
/* 1671 */
pp_task_hdl = 0x3fcdf928;
/* 1672 */
s_pp_task_create_sem = 0x3fcdf924;
/* 1673 */
s_pp_task_del_sem = 0x3fcdf920;
/* 1674 */
g_wifi_menuconfig_ptr = 0x3fcdf91c;
/* 1675 */
xphyQueue = 0x3fcdf918;
/* 1676 */
ap_no_lr_ptr = 0x3fcdf914;
/* 1677 */
rc11BSchedTbl_ptr = 0x3fcdf910;
/* 1678 */
rc11NSchedTbl_ptr = 0x3fcdf90c;
/* 1679 */
rcLoRaSchedTbl_ptr = 0x3fcdf908;
/* 1680 */
BasicOFDMSched_ptr = 0x3fcdf904;
/* 1681 */
trc_ctl_ptr = 0x3fcdf900;
/* 1682 */
g_pm_cnt_ptr = 0x3fcdf8fc;
/* 1683 */
g_pm_ptr = 0x3fcdf8f8;
/* 1684 */
g_pm_cfg_ptr = 0x3fcdf8f4;
/* 1685 */
g_esp_mesh_quick_funcs_ptr = 0x3fcdf8f0;
/* 1686 */
g_txop_queue_status_ptr = 0x3fcdf8ec;
/* 1687 */
g_mac_sleep_en_ptr = 0x3fcdf8e8;
/* 1688 */
g_mesh_is_root_ptr = 0x3fcdf8e4;
/* 1689 */
g_mesh_topology_ptr = 0x3fcdf8e0;
/* 1690 */
g_mesh_init_ps_type_ptr = 0x3fcdf8dc;
/* 1691 */
g_mesh_is_started_ptr = 0x3fcdf8d8;
/* 1692 */
g_config_func = 0x3fcdf8d4;
/* 1693 */
g_net80211_tx_func = 0x3fcdf8d0;
/* 1694 */
g_timer_func = 0x3fcdf8cc;
/* 1695 */
s_michael_mic_failure_cb = 0x3fcdf8c8;
/* 1696 */
wifi_sta_rx_probe_req = 0x3fcdf8c4;
/* 1697 */
g_tx_done_cb_func = 0x3fcdf8c0;
/* 1698 */
g_per_conn_trc = 0x3fcdf874;
/* 1699 */
s_encap_amsdu_func = 0x3fcdf870;


/***************************************
 Group rom_net80211
 ***************************************/

/* Functions */
/* 1707 */
esp_net80211_rom_version_get = 0x40001820;
/* 1736 */
roundup2 = 0x40001894;

/* 1742 */
/* Data (.data, .bss, .rodata) */
/* 1743 */
net80211_funcs = 0x3fcdf86c;
/* 1744 */
g_scan = 0x3fcdf868;
/* 1745 */
g_chm = 0x3fcdf864;
/* 1746 */
g_ic_ptr = 0x3fcdf860;
/* 1747 */
g_hmac_cnt_ptr = 0x3fcdf85c;
/* 1748 */
g_tx_cacheq_ptr = 0x3fcdf858;
/* 1749 */
s_netstack_free = 0x3fcdf854;
/* 1750 */
mesh_rxcb = 0x3fcdf850;
/* 1751 */
sta_rxcb = 0x3fcdf84c;


/* 1788 */
/***************************************
 Group rom_phy
 ***************************************/

/* Functions */
/* 1815 */
rom_i2c_readReg = 0x40001954;
/* 1817 */
rom_i2c_writeReg = 0x4000195c;
/* 1847 */
 rom_read_hw_noisefloor = 0x400019d4;
