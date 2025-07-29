.libphy : ALIGN(4)
{
    /* phy_analog_cal.o */
        *(.text.get_rc_dout)
        *(.text.rc_cal)

    /* phy_api.o */
        *(.text.phy_change_channel)
        *(.text.phy_bbpll_en_usb)
        *(.text.phy_get_rf_cal_version)
        *(.text.noise_check_loop)

    /* phy_basic.o */
        *(.text.chan14_mic_cfg)

    /* phy_debug.o */
        *(.text.get_phy_version_str)
        *(.text.get_iq_value)
        *(.text.get_bias_ref_code)
        *(.text.phy_get_vdd33)

    /* phy_feature.o */
        *(.text.phy_set_most_tpw)
        *(.text.phy_11p_set)
        *(.text.phy_enable_low_rate)
        *(.text.phy_disable_low_rate)
        *(.text.set_rx_sense)

    /* phy_hw_freq.o */
        *(.text.wr_rf_freq_mem)
        *(.text.freq_i2c_write_set)
        *(.text.get_rf_freq_init)
        *(.text.freq_get_i2c_data)
        *(.text.freq_i2c_data_write)
        *(.text.set_chan_freq_hw_init)
        *(.text.set_chan_freq_sw_start)

    /* phy_i2c.o */
        *(.text.rom_i2c_sar2_init_code)
        *(.text.phy_i2c_init2)
        *(.text.phy_get_i2c_data)
        *(.text.bias_reg_set)
        *(.text.i2c_bbpll_set)

    /* phy_init.o */
        *(.text.phy_get_romfunc_addr)
        *(.text.rf_init)
        *(.text.register_chipv7_phy_init_param)
        *(.text.phy_set_mac_data)
        *(.text.phy_rfcal_data_sub)
        *(.text.rf_cal_data_recovery)
        *(.text.phy_rfcal_data_check_value)
        *(.text.rf_cal_data_backup)
        *(.text.phy_rfcal_data_check)
        *(.text.rf_cal_level_check)
        *(.text.bb_init)
        *(.text.register_chipv7_phy)
        *(.text.get_txcap_data)

    /* phy_pbus.o */
        *(.text.ram_pbus_force_mode)
        *(.text.rom_pbus_xpd_tx_on)
        *(.text.txcal_debuge_mode)
        *(.text.txcal_work_mode)
        *(.text.save_pbus_reg)
        *(.text.set_pbus_mem)

    /* phy_pwdet.o */
        *(.text.phy_set_pwdet_power)
        *(.text.get_sar_sig_ref)
        *(.text.pwdet_tone_start)
        *(.text.get_tone_sar_dout)
        *(.text.get_fm_sar_dout)
        *(.text.txtone_linear_pwr)
        *(.text.get_power_db)
        *(.text.rom1_read_sar2_code)

    /* phy_reg.o */
        *(.text.txiq_set_reg)
        *(.text.rxiq_set_reg)
        *(.text.start_tx_tone_step)
        *(.text.stop_tx_tone)
        *(.text.read_hw_noisefloor)
        *(.text.rom1_set_noise_floor)
        *(.text.phy_freq_correct)
        *(.text.force_txrx_off)

    /* phy_rfpll.o */
        *(.text.restart_cal)
        *(.text.write_rfpll_sdm)
        *(.text.wait_rfpll_cal_end)
        *(.text.rfpll_set_freq)
        *(.text.correct_rfpll_offset)
        *(.text.rom2_write_pll_cap)
        *(.text.rom2_read_pll_cap)
        *(.text.rom2_rfpll_cap_correct)
        *(.text.rfpll_cap_init_cal)
        *(.text.set_rfpll_freq)
        *(.text.set_rf_freq_offset)
        *(.text.set_channel_rfpll_freq)
        *(.text.chip_v7_set_chan_misc)
        *(.text.chip_v7_set_chan)
        *(.text.chip_v7_set_chan_offset)
        *(.text.chip_v7_set_chan_ana)
        *(.text.set_chanfreq)

    /* phy_rom.o */
        *(.text.rom_phy_param_addr)
        *(.text.chip726_phyrom_version_num)

    /* phy_rx_cal.o */
        *(.text.rxiq_get_mis)
        *(.text.rxiq_cover_mg_mp)
        *(.text.rfcal_rxiq)
        *(.text.get_rfcal_rxiq_data)
        *(.text.pbus_rx_dco_cal)
        *(.text.rxdc_est_min_new)
        *(.text.pbus_rx_dco_cal_1step_new)
        *(.text.set_rx_gain_cal_iq)
        *(.text.rx_chan_dc_sort)
        *(.text.set_rx_gain_cal_dc)

    /* phy_rx_gain.o */
        *(.text.gen_rx_gain_table)
        *(.text.wr_rx_gain_mem)
        *(.text.set_rx_gain_param)
        *(.text.set_rx_gain_table)

    /* phy_tester_cali.o */

    /* phy_track.o */
        *(.text.rom1_txpwr_cal_track)
        *(.text.txpwr_offset)

    /* phy_tsens.o */
        *(.text.phy_set_tsens_power)
        *(.text.rom2_tsens_read_init1)
        *(.text.tsens_dac_to_index)
        *(.text.tsens_dac_cal1)
        *(.text.rom_tsens_code_read)
        *(.text.tsens_temp_read1)
        *(.text.phy_get_tsens_value)
        *(.text.rom1_tsens_temp_read)
        *(.text.rom2_temp_to_power1)
        *(.text.get_temp_init)

    /* phy_tx_cal.o */
        *(.text.txdc_cal_v70)
        *(.text.bt_txdc_cal)
        *(.text.txdc_cal_init)
        *(.text.txiq_get_mis_pwr)
        *(.text.txiq_cover)
        *(.text.get_power_atten)
        *(.text.rfcal_txiq)
        *(.text.bt_txiq_cal)
        *(.text.txiq_cal_init)
        *(.text.pwdet_ref_code)
        *(.text.pwdet_code_cal)
        *(.text.rfcal_txcap)
        *(.text.tx_cap_init)
        *(.text.rfcal_pwrctrl)
        *(.text.tx_pwctrl_init_cal)
        *(.text.tx_pwctrl_init)
        *(.text.bt_tx_pwctrl_init)
        *(.text.bt_txpwr_freq)

    /* phy_tx_gain.o */
        *(.text.rom_txbbgain_to_index)
        *(.text.rom_index_to_txbbgain)
        *(.text.bt_chan_pwr_interp)
        *(.text.rom_set_tx_gain_mem)
        *(.text.rom1_get_rate_fcc_index)
        *(.text.rom1_get_chan_target_power)
        *(.text.rom2_get_tx_gain_value1)
        *(.text.rom1_wifi_get_tx_gain)
        *(.text.ram1_wifi_set_tx_gain)
        *(.text.rom1_bt_get_tx_gain)
        *(.text.rom1_bt_set_tx_gain)
        *(.text.bt_tx_gain_init)
        *(.text.txcal_gain_check)

} > ROTEXT