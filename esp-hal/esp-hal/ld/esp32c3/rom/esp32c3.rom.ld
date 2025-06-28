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
/* 1504 */
RC_GetBlockAckTime = 0x400015b4;
/* 1506 */
/*esf_buf_alloc = 0x400015bc;*/
/* 1507 */
GetAccess = 0x400015c8;
/* 1508 */
hal_mac_is_low_rate_enabled = 0x400015cc;
/* 1509 */
hal_mac_tx_get_blockack = 0x400015d0;
/* 1510 */
/* hal_mac_tx_set_ppdu = 0x400015d4;*/
/* 1511 */
ic_get_trc = 0x400015d8;
/* 1512 */
/* ic_mac_deinit = 0x400015dc; */
/* 1513 */
ic_mac_init = 0x400015e0;
/* 1514 */
ic_interface_enabled = 0x400015e4;
/* 1515 */
is_lmac_idle = 0x400015e8;
/* 1516 */
/*lmacAdjustTimestamp = 0x400015ec;*/
/* 1517 */
lmacDiscardAgedMSDU = 0x400015f0;
/* 1518 */
/*lmacDiscardMSDU = 0x400015f4;*/
/* 1519 */
/*lmacEndFrameExchangeSequence = 0x400015f8;*/
/* 1520 */
lmacIsIdle = 0x400015fc;
/* 1521 */
lmacIsLongFrame = 0x40001600;
/* 1522 */
/*lmacMSDUAged = 0x40001604;*/
/* 1527 */
lmacReachLongLimit = 0x40001618;
/* 1528 */
lmacReachShortLimit = 0x4000161c;
/* 1529 */
lmacRecycleMPDU = 0x40001620;
/* 1530 */
lmacRxDone = 0x40001624;
/* 1531 */
/*lmacSetTxFrame = 0x40001628;*/
/* 1532 */
/*lmacTxFrame = 0x40001630;*/
/* 1538 */
/* pm_check_state = 0x40001648; */
/* 1539 */
pm_disable_dream_timer = 0x4000164c;
/* 1540 */
pm_disable_sleep_delay_timer = 0x40001650;
/* 1541 */
/*pm_dream = 0x40001654;*/
/* 1542 */
pm_mac_wakeup = 0x40001658;
/* 1543 */
pm_mac_sleep = 0x4000165c;
/* 1544 */
pm_enable_active_timer = 0x40001660;
/* 1545 */
pm_enable_sleep_delay_timer = 0x40001664;
/* 1546 */
pm_local_tsf_process = 0x40001668;
/* 1547 */
pm_set_beacon_filter = 0x4000166c;
/* 1548 */
pm_is_in_wifi_slice_threshold = 0x40001670;
/* 1549 */
pm_is_waked = 0x40001674;
/* 1550 */
pm_keep_alive = 0x40001678;
/* 1551 */
/* pm_on_beacon_rx = 0x4000167c; */
/* 1552 */
pm_on_data_rx = 0x40001680;
/* 1553 */
pm_on_tbtt = 0x40001684;
/* 1554 */
/* pm_parse_beacon = 0x40001688;*/
/* 1555 */
/* pm_process_tim = 0x4000168c; */
/* 1556 */
/*pm_rx_beacon_process = 0x40001690;*/
/* 1557 */
/* pm_rx_data_process = 0x40001694; */
/* 1558 */
/*pm_sleep = 0x40001698;*/
/* 1559 */
pm_sleep_for = 0x4000169c;
/* 1560 */
/* pm_tbtt_process = 0x400016a0; */
/* 1561 */
ppAMPDU2Normal = 0x400016a4;
/* 1562 */
/*ppAssembleAMPDU = 0x400016a8;*/
/* 1563 */
ppCalFrameTimes = 0x400016ac;
/* 1564 */
ppCalSubFrameLength = 0x400016b0;
/* 1565 */
/*ppCalTxAMPDULength = 0x400016b4;*/
/* 1566 */
ppCheckTxAMPDUlength = 0x400016b8;
/* 1567 */
ppDequeueRxq_Locked = 0x400016bc;
/* 1568 */
ppDequeueTxQ = 0x400016c0;
/* 1569 */
ppEmptyDelimiterLength = 0x400016c4;
/* 1570 */
ppEnqueueRxq = 0x400016c8;
/* 1571 */
ppEnqueueTxDone = 0x400016cc;
/* 1573 */
ppGetTxframe = 0x400016d4;
/* 1574 */
ppProcessRxPktHdr = 0x400016e0;
/* 1575 */
ppProcessTxQ = 0x400016e4;
/* 1576 */
ppRecordBarRRC = 0x400016e8;
/* 1577 */
lmacRequestTxopQueue = 0x400016ec;
/* 1578 */
lmacReleaseTxopQueue = 0x400016f0;
/* 1579 */
ppRecycleAmpdu = 0x400016f4;
/* 1580 */
ppRecycleRxPkt = 0x400016f8;
/* 1581 */
ppResortTxAMPDU = 0x400016fc;
/* 1582 */
ppResumeTxAMPDU = 0x40001700;
/* 1583 */
/* ppRxFragmentProc = 0x40001704; */
/* 1584 */
/* ppRxPkt = 0x40001708; */
/* 1585 */
ppRxProtoProc = 0x4000170c;
/* 1586 */
ppSearchTxQueue = 0x40001710;
/* 1587 */
ppSearchTxframe = 0x40001714;
/* 1588 */
ppSelectNextQueue = 0x40001718;
/* 1589 */
ppSubFromAMPDU = 0x4000171c;
/* 1590 */
ppTask = 0x40001720;
/* 1591 */
ppTxPkt = 0x40001724;
/* 1592 */
ppTxProtoProc = 0x40001728;
/* 1593 */
ppTxqUpdateBitmap = 0x4000172c;
/* 1594 */
/*pp_coex_tx_request = 0x40001730;*/
/* 1595 */
pp_hdrsize = 0x40001734;
/* 1596 */
pp_post = 0x40001738;
/* 1597 */
pp_process_hmac_waiting_txq = 0x4000173c;
/* 1598 */
rcGetAmpduSched = 0x40001740;
/* 1599 */
rcUpdateRxDone = 0x40001744;
/* 1600 */
rc_get_trc = 0x40001748;
/* 1601 */
rc_get_trc_by_index = 0x4000174c;
/* 1602 */
rcAmpduLowerRate = 0x40001750;
/* 1604 */
rcClearCurAMPDUSched = 0x40001758;
/* 1605 */
rcClearCurSched = 0x4000175c;
/* 1606 */
rcClearCurStat = 0x40001760;
/*1607 */
rcLowerSched = 0x40001768;
/* 1608 */
rcSetTxAmpduLimit = 0x4000176c;
/* 1609 */
/* rcTxUpdatePer = 0x40001770;*/
/* 1610 */
rcUpdateAckSnr = 0x40001774;
/* 1611 */
/*rcUpdateRate = 0x40001778;*/
/* 1612 */
/* rcUpdateTxDone = 0x4000177c; */
/* 1613 */
rcUpdateTxDoneAmpdu2 = 0x40001780;
/* 1614 */
rcUpSched = 0x40001784;
/* 1615 */
rssi_margin = 0x40001788;
/* 1616 */
rx11NRate2AMPDULimit = 0x4000178c;
/* 1617 */
TRC_AMPDU_PER_DOWN_THRESHOLD = 0x40001790;
/* 1618 */
TRC_AMPDU_PER_UP_THRESHOLD = 0x40001794;
/* 1619 */
trc_calc_duration = 0x40001798;
/* 1620 */
trc_isTxAmpduOperational = 0x4000179c;
/* 1621 */
trc_onAmpduOp = 0x400017a0;
/* 1622 */
TRC_PER_IS_GOOD = 0x400017a4;
/* 1623 */
trc_SetTxAmpduState = 0x400017a8;
/* 1624 */
trc_tid_isTxAmpduOperational = 0x400017ac;
/* 1625 */
trcAmpduSetState = 0x400017b0;
/* 1626 */
/*wDev_AppendRxBlocks = 0x400017b8;*/
/* 1627 */
wDev_DiscardFrame = 0x400017bc;
/* 1631 */
wdev_bank_store = 0x400017cc;
/* 1632 */
wdev_bank_load = 0x400017d0;
/* 1633 */
wdev_mac_reg_load = 0x400017d4;
/* 1634 */
wdev_mac_reg_store = 0x400017d8;
/* 1635 */
wdev_mac_special_reg_load = 0x400017dc;
/* 1636 */
wdev_mac_special_reg_store = 0x400017e0;
/* 1637 */
wdev_mac_wakeup = 0x400017e4;
/* 1638 */
wdev_mac_sleep = 0x400017e8;
/* 1639 */
hal_mac_is_dma_enable = 0x400017ec;
/* 1640 */
/*wDev_ProcessFiq = 0x400017f0;*/
/* 1641 */
/*wDev_ProcessRxSucData = 0x400017f4;*/
/* 1642 */
wdevProcessRxSucDataAll = 0x400017f8;
/* 1643 */
wdev_csi_len_align = 0x400017fc;
/* 1644 */
ppDequeueTxDone_Locked = 0x40001800;
/* 1645 */
/*pm_tx_data_done_process = 0x40001808;*/
/* 1646 */
config_is_cache_tx_buf_enabled = 0x4000180c;
/* 1647 */
//ppMapWaitTxq = 0x40001810;
/* 1648 */
ppProcessWaitingQueue = 0x40001814;
/* 1649 */
ppDisableQueue = 0x40001818;

/* 1651 */
/* Data (.data, .bss, .rodata) */
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
/* 1708 */
ampdu_dispatch = 0x40001824;
/* 1709 */
ampdu_dispatch_all = 0x40001828;
/* 1710 */
ampdu_dispatch_as_many_as_possible = 0x4000182c;
/* 1711 */
ampdu_dispatch_movement = 0x40001830;
/* 1712 */
ampdu_dispatch_upto = 0x40001834;
/* 1713 */
chm_is_at_home_channel = 0x40001838;
/* 1714 */
cnx_node_is_existing = 0x4000183c;
/* 1715 */
cnx_node_search = 0x40001840;
/* 1716 */
ic_ebuf_recycle_rx = 0x40001844;
/* 1717 */
ic_ebuf_recycle_tx = 0x40001848;
/* 1718 */
ic_reset_rx_ba = 0x4000184c;
/* 1719 */
ieee80211_align_eb = 0x40001850;
/* 1720 */
/*ieee80211_ampdu_reorder = 0x40001854;*/
/* 1721 */
ieee80211_ampdu_start_age_timer = 0x40001858;
/* 1722 */
/*ieee80211_encap_esfbuf = 0x4000185c;*/
/* 1723 */
ieee80211_is_tx_allowed = 0x40001860;
/* 1724 */
ieee80211_output_pending_eb = 0x40001864;
/* 1725 */
/*ieee80211_output_process = 0x40001868;*/
/* 1726 */
ieee80211_set_tx_desc = 0x4000186c;
/* 1728 */
wifi_get_macaddr = 0x40001874;
/* 1729 */
wifi_rf_phy_disable = 0x40001878;
/* 1730 */
wifi_rf_phy_enable = 0x4000187c;
/* 1731 */
ic_ebuf_alloc = 0x40001880;
/* 1732 */
/*ieee80211_classify = 0x40001884;*/
/* 1733 */
ieee80211_copy_eb_header = 0x40001888;
/* 1734 */
ieee80211_recycle_cache_eb = 0x4000188c;
/* 1735 */
ieee80211_search_node = 0x40001890;
/* 1736 */
roundup2 = 0x40001894;
/* 1737 */
ieee80211_crypto_encap = 0x40001898;
/* 1738 */
/* ieee80211_crypto_decap = 0x4000189c; */
/* 1739 */
/* ieee80211_decap = 0x400018a0; */
/* 1740 */
ieee80211_set_tx_pti = 0x400018a4;
/* 1741 */
wifi_is_started = 0x400018a8;

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
