.libwpa-suppl : ALIGN(4)
{
    /* os_xtensa.c.obj */

    /* eloop.c.obj */

    /* ap_config.c.obj */

    /* ieee802_1x.c.obj */

    /* wpa_auth.c.obj */

    /* wpa_auth_ie.c.obj */

    /* pmksa_cache_auth.c.obj */

    /* sta_info.c.obj */

    /* ieee802_11.c.obj */

    /* comeback_token.c.obj */

    /* sae.c.obj */

    /* dragonfly.c.obj */

    /* wpa_common.c.obj */

    /* bitfield.c.obj */

    /* aes-siv.c.obj */

    /* sha256-kdf.c.obj */

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

    /* wpa.c.obj */

    /* wpa_ie.c.obj */

    /* base64.c.obj */

    /* common.c.obj */
        *(.text.hex2num)
        *(.text.hex2byte)
        *(.text.hexstr2bin)
        *(.text.__hide_aliasing_typecast)

    /* ext_password.c.obj */

    /* uuid.c.obj */

    /* wpabuf.c.obj */

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

    /* esp_wpa2_api_port.c.obj */

    /* esp_wpa_main.c.obj */

    /* esp_wpas_glue.c.obj */

    /* esp_common.c.obj */

    /* esp_wps.c.obj */

    /* esp_wpa3.c.obj */

    /* esp_owe.c.obj */

    /* esp_hostap.c.obj */

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