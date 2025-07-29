.libwpa-suppl : ALIGN(4)
{
    /* os_xtensa.c.obj */
        *(.text.os_get_time)
        *(.text.os_get_random)
        *(.text.os_sleep)

    /* eloop.c.obj */
        *(.text.eloop_run_timer)
        *(.text.eloop_remove_timeout)
        *(.text.os_timer_disarm.constprop.0)
        *(.text.eloop_init)
        *(.text.eloop_register_timeout)
        *(.text.eloop_cancel_timeout)
        *(.text.eloop_run)
        *(.text.eloop_run_wrapper)

    /* ap_config.c.obj */
        *(.text.hostapd_setup_wpa_psk)
        *(.text.hostapd_get_psk)
        *(.text.hostapd_config_clear_wpa_psk)
        *(.text.hostapd_config_free_bss)

    /* ieee802_1x.c.obj */

    /* wpa_auth.c.obj */
        *(.text.wpa_auth_pmksa_free_cb)
        *(.text.wpa_free_sta_sm)
        *(.text.wpa_group_init_gmk_and_counter)
        *(.text.wpa_gmk_to_gtk)
        *(.text.wpa_replay_counter_valid)
        *(.text.wpa_replay_counter_mark_invalid)
        *(.text.wpa_verify_key_mic)
        *(.text.ieee80211w_kde_add)
        *(.text.resend_eapol_handle)
        *(.text.wpa_auth_set_key.part.0)
        *(.text.sm_WPA_PTK_DISCONNECT_Enter.constprop.0)
        *(.text.wpa_gtk_update.isra.0)
        *(.text.wpa_group_gtk_init)
        *(.text.wpa_auth_get_psk.isra.0)
        *(.text.sm_WPA_PTK_PTKCALCNEGOTIATING_Enter.constprop.0)
        *(.text.wpa_group_config_group_keys)
        *(.text.sm_WPA_PTK_AUTHENTICATION2_Enter.constprop.0)
        *(.text.wpa_group_sm_step.part.0)
        *(.text.wpa_rekey_gtk)
        *(.text.wpa_init)
        *(.text.wpa_auth_sta_init)
        *(.text.wpa_auth_sta_deinit)
        *(.text.__wpa_send_eapol)
        *(.text.wpa_send_eapol)
        *(.text.sm_WPA_PTK_PTKSTART_Enter.constprop.0)
        *(.text.sm_WPA_PTK_PTKINITNEGOTIATING_Enter.constprop.0)
        *(.text.sm_WPA_PTK_GROUP_REKEYNEGOTIATING_Enter.constprop.0)
        *(.text.wpa_remove_ptk)
        *(.text.sm_WPA_PTK_INITIALIZE_Enter.constprop.0)
        *(.text.wpa_sm_step)
        *(.text.wpa_rekey_ptk)
        *(.text.wpa_auth_sta_associated)
        *(.text.wpa_receive)
        *(.text.hostap_eapol_resend_process)
        *(.text.wpa_deinit)
        *(.text.wpa_ap_join)
        *(.text.wpa_ap_remove)

    /* wpa_auth_ie.c.obj */
        *(.text.wpa_write_rsn_ie)
        *(.text.wpa_write_rsnxe)
        *(.text.wpa_auth_gen_wpa_ie)
        *(.text.wpa_add_kde)
        *(.text.wpa_validate_wpa_ie)
        *(.text.wpa_parse_kde_ies)
        *(.text.wpa_auth_uses_mfp)

    /* pmksa_cache_auth.c.obj */
        *(.text._pmksa_cache_free_entry)
        *(.text.pmksa_cache_auth_deinit)
        *(.text.pmksa_cache_auth_get)
        *(.text.pmksa_cache_auth_init)

    /* sta_info.c.obj */
        *(.text.ap_get_sta)
        *(.text.ap_sta_hash_add)
        *(.text.ap_free_sta)
        *(.text.ap_sta_add)

    /* ieee802_11.c.obj */
        *(.text.wpa_res_to_status_code)

    /* comeback_token.c.obj */

    /* sae.c.obj */

    /* dragonfly.c.obj */

    /* wpa_common.c.obj */
        *(.text.rsn_selector_to_bitfield)
        *(.text.wpa_selector_to_bitfield)
        *(.text.wpa_mic_len)
        *(.text.wpa_cipher_valid_mgmt_group)
        *(.text.wpa_parse_wpa_ie_rsnxe)
        *(.text.wpa_parse_wpa_ie_rsn)
        *(.text.wpa_parse_wpa_ie_wpa)
        *(.text.wpa_use_akm_defined)
        *(.text.wpa_use_aes_key_wrap)
        *(.text.wpa_eapol_key_mic)
        *(.text.wpa_compare_rsn_ie)
        *(.text.rsn_pmkid)
        *(.text.wpa_cipher_key_len)
        *(.text.wpa_pmk_to_ptk)
        *(.text.wpa_cipher_to_alg)
        *(.text.wpa_cipher_valid_pairwise)
        *(.text.wpa_cipher_to_suite)
        *(.text.rsn_cipher_put_suites)
        *(.text.wpa_cipher_put_suites)

    /* bitfield.c.obj */

    /* aes-siv.c.obj */

    /* sha256-kdf.c.obj */
        *(.text.forced_memzero)

    /* ccmp.c.obj */
        *(.text.ccmp_aad_nonce)
        *(.text.ccmp_decrypt)
        *(.text.ccmp_encrypt)

    /* aes-gcm.c.obj */
        *(.text.WPA_GET_BE32)
        *(.text.WPA_PUT_BE32)
        *(.text.WPA_PUT_BE64)
        *(.text.xor_block)
        *(.text.shift_right_block)
        *(.text.gf_mult)
        *(.text.aes_gcm_init_hash_subkey)
        *(.text.ghash)
        *(.text.aes_gcm_ghash)
        *(.text.aes_gctr.part.0)
        *(.text.aes_gcm_prepare_j0)
        *(.text.aes_gcm_gctr.part.0)
        *(.text.aes_gcm_ae)
        *(.text.aes_gmac)


    /* crypto_ops.c.obj */
        *(.text.esp_supp_crc32)
        *(.text.esp_aes_gmac)
        *(.text.esp_aes_decrypt)
        *(.text.esp_aes_encrypt)
        *(.text.esp_aes_unwrap)
        *(.text.esp_aes_wrap)

    /* dh_group5.c.obj */

    /* dh_groups.c.obj */

    /* ms_funcs.c.obj */

    /* sha1-tlsprf.c.obj */

    /* sha256-tlsprf.c.obj */

    /* sha384-tlsprf.c.obj */

    /* sha256-prf.c.obj */
        *(.text.sha256_prf_bits)
        *(.text.sha256_prf)

    /* sha1-prf.c.obj */
        *(.text.sha1_prf)

    /* sha384-prf.c.obj */

    /* md4-internal.c.obj */

    /* sha1-tprf.c.obj */

    /* eap_wsc_common.c.obj */

    /* ieee802_11_common.c.obj */

    /* chap.c.obj */

    /* eap.c.obj */

    /* eap_common.c.obj */

    /* eap_mschapv2.c.obj */

    /* eap_peap.c.obj */

    /* eap_peap_common.c.obj */

    /* eap_tls.c.obj */

    /* eap_tls_common.c.obj */

    /* eap_ttls.c.obj */

    /* mschapv2.c.obj */

    /* eap_fast.c.obj */

    /* eap_fast_common.c.obj */

    /* eap_fast_pac.c.obj */

    /* pmksa_cache.c.obj */
        *(.text.pmksa_cache_set_expiration)
        *(.text.pmksa_cache_free_entry)
        *(.text.pmksa_cache_expire)
        *(.text.pmksa_cache_flush)
        *(.text.pmksa_cache_add_entry)
        *(.text.pmksa_cache_add)
        *(.text.pmksa_cache_deinit)
        *(.text.pmksa_cache_get)
        *(.text.pmksa_cache_get_opportunistic)
        *(.text.pmksa_cache_clear_current)
        *(.text.pmksa_cache_set_current)
        *(.text.pmksa_cache_init)

    /* wpa.c.obj */
        *(.text.is_wpa2_enterprise_connection)
        *(.text.wpa_sm_pmksa_free_cb)
        *(.text.wpa_sm_set_seq.constprop.0)
        *(.text.ieee80211w_set_keys.constprop.0)
        *(.text.cipher_type_map_supp_to_public)
        *(.text.cipher_type_map_public_to_supp)
        *(.text.wpa_eapol_key_send)
        *(.text.wpa_sm_key_request)
        *(.text.wpa_sm_rekey_ptk)
        *(.text.wpa_supplicant_send_2_of_4)
        *(.text.wpa_supplicant_gtk_tx_bit_workaround)
        *(.text.wpa_supplicant_pairwise_gtk)
        *(.text.wpa_sm_set_state)
        *(.text.wpa_supplicant_key_neg_complete)
        *(.text.wpa_supplicant_stop_countermeasures)
        *(.text.wpa_sm_set_pmk_from_pmksa)
        *(.text.wpa_supplicant_process_1_of_4)
        *(.text.wpa_sm_init)
        *(.text.wpa_sm_notify_assoc)
        *(.text.wpa_set_profile)
        *(.text.wpa_set_passphrase)
        *(.text.set_assoc_ie)
        *(.text.wpa_sm_set_key)
        *(.text.wpa_supplicant_install_gtk.constprop.0)
        *(.text.wpa_supplicant_install_ptk.constprop.0)
        *(.text.wpa_sm_rx_eapol)
        *(.text.wpa_michael_mic_failure)
        *(.text.eapol_txcb)
        *(.text.wpa_sta_in_4way_handshake)
        *(.text.wpa_sta_clear_curr_pmksa)
        *(.text.wpa_sm_set_ap_rsnxe)
        *(.text.wpa_sm_set_assoc_rsnxe)
        *(.text.wpa_set_bss)
        *(.text.wpa_sm_drop_sa)
        *(.text.wpa_sm_deinit)
        *(.text.wpa_sm_notify_disassoc)

    /* wpa_ie.c.obj */
        *(.text.wpa_parse_wpa_ie)
        *(.text.wpa_gen_wpa_ie)
        *(.text.wpa_gen_rsnxe)
        *(.text.wpa_supplicant_parse_ies)

    /* base64.c.obj */

    /* common.c.obj */
        *(.text.hex2num)
        *(.text.hex2byte)
        *(.text.hexstr2bin)
        *(.text.inc_byte_array)
        *(.text.wpa_get_ntp_timestamp)
        *(.text.printf_decode)
        *(.text.__hide_aliasing_typecast)
        *(.text.dup_binstr)
        *(.text.wpa_config_parse_string)
        *(.text.bin_clear_free)
        *(.text.os_memdup)

    /* ext_password.c.obj */

    /* uuid.c.obj */

    /* wpabuf.c.obj */
        *(.text.wpabuf_free)

    /* wpa_debug.c.obj */

    /* json.c.obj */

    /* wps.c.obj */

    /* wps_attr_build.c.obj */

    /* wps_attr_parse.c.obj */

    /* wps_attr_process.c.obj */

    /* wps_common.c.obj */

    /* wps_dev_attr.c.obj */

    /* wps_enrollee.c.obj */

    /* esp_eap_client.c.obj */
        *(.text.eap_client_get_eap_state)

    /* esp_wpa2_api_port.c.obj */

    /* esp_wpa_main.c.obj */
        *(.text.wpa_ap_get_peer_spp_msg)
        *(.text.wpa_attach)
        *(.text.wpa_config_done)
        *(.text.wpa_ap_get_wpa_ie)
        *(.text.wpa_deattach)
        *(.text.wpa_parse_wpa_ie_wrapper)
        *(.text.hostap_sta_join)
        *(.text.wpa_sta_disconnected_cb)
        *(.text.wpa_sta_connected_cb)
        *(.text.wpa_ap_rx_eapol)
        *(.text.wpa_install_key)
        *(.text.wpa_get_key)
        *(.text.wpa_deauthenticate)
        *(.text.wpa_config_profile)
        *(.text.wpa_config_bss)
        *(.text.wpa_sta_connect)
        *(.text.wpa_config_assoc_ie)
        *(.text.wpa_neg_complete)
        *(.text.esp_supplicant_init)

    /* esp_wpas_glue.c.obj */
        *(.text.wpa_alloc_eapol)
        *(.text.wpa_free_eapol)
        *(.text.wpa_ether_send)
        *(.text.hostapd_send_eapol)
        *(.text.wpa_supplicant_transition_disable)
        *(.text.wpa_sm_alloc_eapol)
        *(.text.wpa_sm_free_eapol)
        *(.text.wpa_sm_deauthenticate)
        *(.text.wpa_sm_mlme_setprotection)
        *(.text.wpa_sm_disassociate)

    /* esp_common.c.obj */
        *(.text.ieee80211_handle_rx_frm)
        *(.text.esp_supplicant_common_deinit)
        *(.text.esp_supplicant_common_init)
        *(.text.supplicant_sta_conn_handler)
        *(.text.supplicant_sta_disconn_handler)
        *(.text.esp_set_scan_ie)
        *(.text.esp_set_assoc_ie)

    /* esp_wps.c.obj */
        *(.text.wps_get_wps_sm_cb)

    /* esp_wpa3.c.obj */

    /* esp_owe.c.obj */

    /* esp_hostap.c.obj */
        *(.text.hostapd_get_hapd_data)
        *(.text.hostap_init)
        *(.text.hostapd_cleanup)
        *(.text.hostap_deinit)
        *(.text.esp_wifi_build_rsnxe)
        *(.text.esp_send_assoc_resp)

    /* asn1.c.obj */

    /* bignum.c.obj */

    /* pkcs1.c.obj */

    /* pkcs5.c.obj */

    /* pkcs8.c.obj */

    /* rsa.c.obj */

    /* tls_internal.c.obj */

    /* tlsv1_client.c.obj */

    /* tlsv1_client_read.c.obj */

    /* tlsv1_client_write.c.obj */

    /* tlsv1_common.c.obj */

    /* tlsv1_cred.c.obj */

    /* tlsv1_record.c.obj */

    /* tlsv1_client_ocsp.c.obj */

    /* x509v3.c.obj */

    /* rc4.c.obj */
        *(.text.rc4_skip)

    /* aes-ctr.c.obj */

    /* aes-cbc.c.obj */
        *(.text.aes_128_cbc_encrypt)
        *(.text.aes_128_cbc_decrypt)

    /* aes-ccm.c.obj */
        *(.text.xor_aes_block)
        *(.text.aes_ccm_auth)
        *(.text.aes_ccm_encr.constprop.0)
        *(.text.aes_ccm_auth_start.constprop.0)
        *(.text.aes_ccm_ae)
        *(.text.aes_ccm_ad)

    /* aes-internal-dec.c.obj */
        *(.text.aes_decrypt_init)
        *(.text.aes_decrypt)
        *(.text.aes_decrypt_deinit)

    /* aes-internal-enc.c.obj */
        *(.text.aes_encrypt_init)
        *(.text.aes_encrypt)
        *(.text.aes_encrypt_deinit)

    /* aes-internal.c.obj */
        *(.text.rijndaelKeySetupEnc)

    /* aes-omac1.c.obj */
        *(.text.gf_mulx)
        *(.text.omac1_aes_vector)
        *(.text.omac1_aes_128_vector)
        *(.text.omac1_aes_128)

    /* aes-unwrap.c.obj */
        *(.text.aes_unwrap)

    /* aes-wrap.c.obj */
        *(.text.aes_wrap)

    /* crypto_internal-cipher.c.obj */

    /* crypto_internal-modexp.c.obj */

    /* crypto_internal-rsa.c.obj */

    /* crypto_internal.c.obj */

    /* des-internal.c.obj */

    /* md5-internal.c.obj */
        *(.text.MD5Transform)
        *(.text.MD5Init)
        *(.text.MD5Update)
        *(.text.MD5Final)
        *(.text.md5_vector)

    /* md5.c.obj */
        *(.text.hmac_md5_vector)
        *(.text.hmac_md5)

    /* sha1-internal.c.obj */
         *(.text.SHA1Transform)
        *(.text.SHA1Init)
        *(.text.SHA1Update)
        *(.text.SHA1Final)
        *(.text.sha1_vector)

    /* sha1-pbkdf2.c.obj */
        *(.text.pbkdf2_sha1)

    /* sha1.c.obj */
        *(.text.hmac_sha1_vector)
        *(.text.hmac_sha1)

    /* sha256-internal.c.obj */
        *(.text.sha256_compress)
        *(.text.sha256_init)
        *(.text.sha256_process)
        *(.text.sha256_done)
        *(.text.sha256_vector)

    /* sha256.c.obj */
        *(.text.hmac_sha256_vector)

    /* sha384-internal.c.obj */

    /* sha512-internal.c.obj */

} > ROTEXT