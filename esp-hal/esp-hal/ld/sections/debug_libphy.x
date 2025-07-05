.libphy : ALIGN(4)
{
    /* phy_analog_cal.o */

    /* phy_api.o */
        *(.text.phy_change_channel)
        *(.text.noise_check_loop)

    /* phy_basic.o */
        *(.text.chan14_mic_cfg)

    /* phy_debug.o */

    /* phy_feature.o */
        *(.text.phy_set_most_tpw)
        *(.text.phy_11p_set)
        *(.text.phy_enable_low_rate)
        *(.text.phy_disable_low_rate)
        *(.text.set_rx_sense)

    /* phy_hw_freq.o */
        *(.text.set_chan_freq_sw_start)

    /* phy_i2c.o */
        *(.text.rom_i2c_sar2_init_code)
        *(.text.phy_i2c_init2)

    /* phy_init.o */
        *(.text.get_txcap_data)

    /* phy_pbus.o */

    /* phy_pwdet.o */

    /* phy_reg.o */
        *(.text.read_hw_noisefloor)
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
        *(.text.set_chanfreq)

    /* phy_rom.o */

    /* phy_rx_cal.o */

    /* phy_rx_gain.o */
        *(.text.wr_rx_gain_mem)

    /* phy_tester_cali.o */

    /* phy_track.o */

    /* phy_tsens.o */

    /* phy_tx_cal.o */

    /* phy_tx_gain.o */
        *(.text.rom_set_tx_gain_mem)
        *(.text.rom1_get_chan_target_power)
        *(.text.rom2_get_tx_gain_value1)
        *(.text.rom1_wifi_get_tx_gain)
        *(.text.ram1_wifi_set_tx_gain)

} > ROTEXT