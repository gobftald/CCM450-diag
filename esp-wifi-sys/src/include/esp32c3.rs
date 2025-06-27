#![allow(non_camel_case_types, non_upper_case_globals)]

// 841
pub const CONFIG_ESP_WIFI_TX_BUFFER_TYPE: u32 = 1;

// 844
pub const CONFIG_ESP_WIFI_DYNAMIC_RX_MGMT_BUF: u32 = 0;
pub const CONFIG_ESP_WIFI_RX_MGMT_BUF_NUM_DEF: u32 = 5;

// 861
pub const CONFIG_ESP_WIFI_ESPNOW_MAX_ENCRYPT_NUM: u32 = 7;

// 1277
pub const ESP_OK: u32 = 0;
pub const ESP_FAIL: i32 = -1;
pub const ESP_ERR_NO_MEM: u32 = 257;
pub const ESP_ERR_INVALID_ARG: u32 = 258;

// 1354
pub const ESP_ERR_WIFI_NOT_INIT: u32 = 12289;
pub const ESP_ERR_WIFI_NOT_STARTED: u32 = 12290;
pub const ESP_ERR_WIFI_NOT_STOPPED: u32 = 12291;
pub const ESP_ERR_WIFI_IF: u32 = 12292;
pub const ESP_ERR_WIFI_MODE: u32 = 12293;
pub const ESP_ERR_WIFI_STATE: u32 = 12294;
pub const ESP_ERR_WIFI_CONN: u32 = 12295;
pub const ESP_ERR_WIFI_NVS: u32 = 12296;
pub const ESP_ERR_WIFI_MAC: u32 = 12297;
pub const ESP_ERR_WIFI_SSID: u32 = 12298;
pub const ESP_ERR_WIFI_PASSWORD: u32 = 12299;
pub const ESP_ERR_WIFI_TIMEOUT: u32 = 12300;
pub const ESP_ERR_WIFI_WAKE_FAIL: u32 = 12301;
pub const ESP_ERR_WIFI_WOULD_BLOCK: u32 = 12302;
pub const ESP_ERR_WIFI_NOT_CONNECT: u32 = 12303;

// 1369
pub const ESP_ERR_WIFI_POST: u32 = 12306;
pub const ESP_ERR_WIFI_INIT_STATE: u32 = 12307;
pub const ESP_ERR_WIFI_STOP_STATE: u32 = 12308;
pub const ESP_ERR_WIFI_NOT_ASSOC: u32 = 12309;
pub const ESP_ERR_WIFI_TX_DISALLOW: u32 = 12310;

// 1381
pub const WIFI_CACHE_TX_BUFFER_NUM: u32 = 0;

// 1390
pub const WIFI_INIT_CONFIG_MAGIC: u32 = 523190095;

// 1393
pub const WIFI_SOFTAP_BEACON_MAX_LEN: u32 = 752;
pub const WIFI_MGMT_SBUF_NUM: u32 = 32;

// 1414
pub const WIFI_FEATURE_CAPS: u32 = 160;

// 1421
pub const ESP_WIFI_OS_ADAPTER_VERSION: u32 = 8;
pub const ESP_WIFI_OS_ADAPTER_MAGIC: u32 = 3735928495;

#[repr(C)]
#[derive(Copy, Clone)]
// 1528
pub struct ets_timer {
    pub next: *mut timer_adpt,
    pub expire: u32,
    pub period: u32,
    pub func: ::core::option::Option<unsafe extern "C" fn(priv_: *mut crate::c_types::c_void)>,
    pub priv_: *mut crate::c_types::c_void,
}

// 1550
pub type va_list = __builtin_va_list;

unsafe extern "C" {
    // 1979
    pub fn puts(arg1: *const crate::c_types::c_char) -> crate::c_types::c_int;
}

unsafe extern "C" {
    // 2037
    pub fn sprintf(
        arg1: *mut crate::c_types::c_char,
        arg2: *const crate::c_types::c_char,
        ...
    ) -> crate::c_types::c_int;
}

unsafe extern "C" {
    // 2072
    pub fn vsnprintf(
        arg1: *mut crate::c_types::c_char,
        arg2: crate::c_types::c_uint,
        arg3: *const crate::c_types::c_char,
        arg4: __builtin_va_list,
    ) -> crate::c_types::c_int;
}

unsafe extern "C" {
    // 3166
    pub fn malloc(arg1: crate::c_types::c_uint) -> *mut crate::c_types::c_void;
}

// 3692
pub type esp_err_t = crate::c_types::c_int;

// 3724
pub type esp_event_base_t = *const crate::c_types::c_char;

/// @brief The AES 128 encrypt callback function used by esp_wifi.
/// @param key  Encryption key.
/// @param iv  Encryption IV for CBC mode (16 bytes).
/// @param data  Data to encrypt in-place.
/// @param data_len  Length of data in bytes (must be divisible by 16)
// 6228
pub type esp_aes_128_encrypt_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        iv: *const crate::c_types::c_uchar,
        data: *mut crate::c_types::c_uchar,
        data_len: crate::c_types::c_int,
    ) -> crate::c_types::c_int,
>;

/// @brief The AES 128 decrypt callback function used by esp_wifi.
/// @param key  Decryption key.
/// @param iv  Decryption IV for CBC mode (16 bytes).
/// @param data  Data to decrypt in-place.
/// @param data_len  Length of data in bytes (must be divisible by 16)\n
// 6237
pub type esp_aes_128_decrypt_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        iv: *const crate::c_types::c_uchar,
        data: *mut crate::c_types::c_uchar,
        data_len: crate::c_types::c_int,
    ) -> crate::c_types::c_int,
>;

/// @brief The AES wrap callback function used by esp_wifi.
///
/// @param kek  16-octet Key encryption key (KEK).
/// @param n  Length of the plaintext key in 64-bit units;
/// @param plain  Plaintext key to be wrapped, n * 64 bits
/// @param cipher  Wrapped key, (n + 1) * 64 bits
///
// 6246
pub type esp_aes_wrap_t = ::core::option::Option<
    unsafe extern "C" fn(
        kek: *const crate::c_types::c_uchar,
        n: crate::c_types::c_int,
        plain: *const crate::c_types::c_uchar,
        cipher: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief The AES unwrap callback function used by esp_wifi.
///
/// @param kek  16-octet Key decryption key (KEK).
/// @param n  Length of the plaintext key in 64-bit units;
/// @param cipher  Wrapped key to be unwrapped, (n + 1) * 64 bits
/// @param plain  Plaintext key, n * 64 bits
// 6255
pub type esp_aes_unwrap_t = ::core::option::Option<
    unsafe extern "C" fn(
        kek: *const crate::c_types::c_uchar,
        n: crate::c_types::c_int,
        cipher: *const crate::c_types::c_uchar,
        plain: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief The SHA256 callback function used by esp_wifi.
///
/// @param key  Key for HMAC operations.
/// @param key_len  Length of the key in bytes.
/// @param num_elem  Number of elements in the data vector.
/// @param addr  Pointers to the data areas.
/// @param len  Lengths of the data blocks.
/// @param mac  Buffer for the hash (32 bytes).
// 6264
pub type esp_hmac_sha256_vector_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_int,
        num_elem: crate::c_types::c_int,
        addr: *mut *const crate::c_types::c_uchar,
        len: *const crate::c_types::c_int,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief The SHA256 PRF callback function used by esp_wifi.
///
/// @param key  Key for PRF.
/// @param key_len  Length of the key in bytes.
/// @param label  A unique label for each purpose of the PRF.
/// @param data  Extra data to bind into the key.
/// @param data_len  Length of the data.
/// @param buf  Buffer for the generated pseudo-random key.
/// @param buf_len  Number of bytes of key to generate.
// 6275
pub type esp_sha256_prf_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_int,
        label: *const crate::c_types::c_char,
        data: *const crate::c_types::c_uchar,
        data_len: crate::c_types::c_int,
        buf: *mut crate::c_types::c_uchar,
        buf_len: crate::c_types::c_int,
    ) -> crate::c_types::c_int,
>;

/// @brief HMAC-MD5 callback function over data buffer (RFC 2104)'
///
/// @param key Key for HMAC operations
/// @param key_len Length of the key in bytes
/// @param data Pointers to the data area
/// @param data_len Length of the data area
/// @param mac Buffer for the hash (16 bytes)
/// Returns: 0 on success, -1 on failure
// 6287
pub type esp_hmac_md5_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_uint,
        data: *const crate::c_types::c_uchar,
        data_len: crate::c_types::c_uint,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief HMAC-MD5 callback function over data vector (RFC 2104)
///
/// @param key Key for HMAC operations
/// @param key_len Length of the key in bytes
/// @param num_elem Number of elements in the data vector
/// @param addr Pointers to the data areas
/// @param len Lengths of the data blocks
/// @param mac Buffer for the hash (16 bytes)
/// Returns: 0 on success, -1 on failure
// 6297
pub type esp_hmac_md5_vector_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_uint,
        num_elem: crate::c_types::c_uint,
        addr: *mut *const crate::c_types::c_uchar,
        len: *const crate::c_types::c_uint,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief HMAC-SHA1 callback function over data buffer (RFC 2104)
///
/// @param key Key for HMAC operations
/// @param key_len Length of the key in bytes
/// @param data Pointers to the data area
/// @param data_len Length of the data area
/// @param mac Buffer for the hash (20 bytes)
/// Returns: 0 on success, -1 of failure
// 6308
pub type esp_hmac_sha1_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_uint,
        data: *const crate::c_types::c_uchar,
        data_len: crate::c_types::c_uint,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief HMAC-SHA1 callback function over data vector (RFC 2104)
///
/// @param key Key for HMAC operations
/// @param key_len Length of the key in bytes
/// @param num_elem Number of elements in the data vector
/// @param addr Pointers to the data areas
/// @param len Lengths of the data blocks
/// @param mac Buffer for the hash (20 bytes)
/// Returns: 0 on success, -1 on failure
// 6318
pub type esp_hmac_sha1_vector_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_uint,
        num_elem: crate::c_types::c_uint,
        addr: *mut *const crate::c_types::c_uchar,
        len: *const crate::c_types::c_uint,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief SHA1-based Pseudo-Random Function (PRF) (IEEE 802.11i, 8.5.1.1) callback function
/// @param key Key for PRF
/// @param key_len Length of the key in bytes
/// @param label A unique label for each purpose of the PRF
/// @param data Extra data to bind into the key
/// @param data_len Length of the data
/// @param buf Buffer for the generated pseudo-random key
/// @param buf_len Number of bytes of key to generate
/// Returns: 0 on success, -1 of failure
///
/// This function is used to derive new, cryptographically separate keys from a
/// given key (e.g., PMK in IEEE 802.11i).
// 6329
pub type esp_sha1_prf_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        key_len: crate::c_types::c_uint,
        label: *const crate::c_types::c_char,
        data: *const crate::c_types::c_uchar,
        data_len: crate::c_types::c_uint,
        buf: *mut crate::c_types::c_uchar,
        buf_len: crate::c_types::c_uint,
    ) -> crate::c_types::c_int,
>;

/// @brief SHA-1 hash callback function for data vector
///
/// @param num_elem Number of elements in the data vector
/// @param addr Pointers to the data areas
/// @param len Lengths of the data blocks
/// @param mac Buffer for the hash
/// Returns: 0 on success, -1 on failure
// 6341
pub type esp_sha1_vector_t = ::core::option::Option<
    unsafe extern "C" fn(
        num_elem: crate::c_types::c_uint,
        addr: *mut *const crate::c_types::c_uchar,
        len: *const crate::c_types::c_uint,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief SHA1-based key derivation function (PBKDF2) callback function for IEEE 802.11i
///
/// @param passphrase ASCII passphrase
/// @param ssid SSID
/// @param ssid_len SSID length in bytes
/// @param iterations Number of iterations to run
/// @param buf Buffer for the generated key
/// @param buflen Length of the buffer in bytes
/// Returns: 0 on success, -1 of failure
///
/// This function is used to derive PSK for WPA-PSK. For this protocol,
/// iterations is set to 4096 and buflen to 32. This function is described in
/// IEEE Std 802.11-2004, Clause H.4. The main construction is from PKCS#5 v2.0.
// 6350
pub type esp_pbkdf2_sha1_t = ::core::option::Option<
    unsafe extern "C" fn(
        passphrase: *const crate::c_types::c_char,
        ssid: *const crate::c_types::c_char,
        ssid_len: crate::c_types::c_uint,
        iterations: crate::c_types::c_int,
        buf: *mut crate::c_types::c_uchar,
        buflen: crate::c_types::c_uint,
    ) -> crate::c_types::c_int,
>;

/// @brief XOR RC4 stream callback function to given data with skip-stream-start
///
/// @param key RC4 key
/// @param keylen RC4 key length
/// @param skip number of bytes to skip from the beginning of the RC4 stream
/// @param data data to be XOR'ed with RC4 stream
/// @param data_len buf length
/// Returns: 0 on success, -1 on failure
///
/// Generate RC4 pseudo random stream for the given key, skip beginning of the
/// stream, and XOR the end result with the data buffer to perform RC4
/// encryption/decryption.
// 6361
pub type esp_rc4_skip_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        keylen: crate::c_types::c_uint,
        skip: crate::c_types::c_uint,
        data: *mut crate::c_types::c_uchar,
        data_len: crate::c_types::c_uint,
    ) -> crate::c_types::c_int,
>;

/// @brief MD5 hash callback function for data vector
/// @param num_elem Number of elements in the data vector
/// @param addr Pointers to the data areas
/// @param len Lengths of the data blocks
/// @param mac Buffer for the hash
/// Returns: 0 on success, -1 on failure
// 6371
pub type esp_md5_vector_t = ::core::option::Option<
    unsafe extern "C" fn(
        num_elem: crate::c_types::c_uint,
        addr: *mut *const crate::c_types::c_uchar,
        len: *const crate::c_types::c_uint,
        mac: *mut crate::c_types::c_uchar,
    ) -> crate::c_types::c_int,
>;

/// @brief Encrypt one AES block callback function
/// @param ctx Context pointer from aes_encrypt_init()
/// @param plain Plaintext data to be encrypted (16 bytes)
/// @param crypt Buffer for the encrypted data (16 bytes)
// 6380
pub type esp_aes_encrypt_t = ::core::option::Option<
    unsafe extern "C" fn(
        ctx: *mut crate::c_types::c_void,
        plain: *const crate::c_types::c_uchar,
        crypt: *mut crate::c_types::c_uchar,
    ),
>;

/// @brief Initialize AES callback function for encryption
/// @param key Encryption key
/// @param len Key length in bytes (usually 16, i.e., 128 bits)
/// Returns: Pointer to context data or %NULL on failure
// 6388
pub type esp_aes_encrypt_init_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        len: crate::c_types::c_uint,
    ) -> *mut crate::c_types::c_void,
>;

/// @brief Deinitialize AES encryption callback function
/// @param ctx Context pointer from aes_encrypt_init()
// 6395
pub type esp_aes_encrypt_deinit_t =
    ::core::option::Option<unsafe extern "C" fn(ctx: *mut crate::c_types::c_void)>;

/// @brief Decrypt one AES block callback function
/// @param ctx Context pointer from aes_encrypt_init()
/// @param crypt Encrypted data (16 bytes)
/// @param plain Buffer for the decrypted data (16 bytes)
// 6398
pub type esp_aes_decrypt_t = ::core::option::Option<
    unsafe extern "C" fn(
        ctx: *mut crate::c_types::c_void,
        crypt: *const crate::c_types::c_uchar,
        plain: *mut crate::c_types::c_uchar,
    ),
>;

/// brief Initialize AES callback function for decryption
/// @param key Decryption key
/// @param len Key length in bytes (usually 16, i.e., 128 bits)
/// Returns: Pointer to context data or %NULL on failure
// 6406
pub type esp_aes_decrypt_init_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const crate::c_types::c_uchar,
        len: crate::c_types::c_uint,
    ) -> *mut crate::c_types::c_void,
>;

/// @brief Deinitialize AES decryption callback function
/// @param ctx Context pointer from aes_encrypt_init()
// 6413
pub type esp_aes_decrypt_deinit_t =
    ::core::option::Option<unsafe extern "C" fn(ctx: *mut crate::c_types::c_void)>;

/// @brief One-Key CBC MAC (OMAC1) hash with AES-128 callback function for MIC computation
///
/// @param key 128-bit key for the hash operation
/// @param data Data buffer for which a MIC is computed
/// @param data_len Length of data buffer in bytes
/// @param mic Buffer for MIC (128 bits, i.e., 16 bytes)
/// Returns: 0 on success, -1 on failure
// 6416
pub type esp_omac1_aes_128_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const u8,
        data: *const u8,
        data_len: usize,
        mic: *mut u8,
    ) -> crate::c_types::c_int,
>;

/// @brief Decrypt data callback function using CCMP (Counter Mode CBC-MAC Protocol OR
///         Counter Mode Cipher Block Chaining Message Authentication
///         Code Protocol) which is used in IEEE 802.11i RSN standard.
/// @param tk 128-bit Temporal Key for obtained during 4-way handshake
/// @param ieee80211_hdr Pointer to IEEE802.11 frame headeri needed for AAD
/// @param data Pointer to encrypted data buffer
/// @param data_len Encrypted data length in bytes
/// @param decrypted_len Length of decrypted data
/// @param espnow_pkt Indicates if it's an ESPNOW packet
/// Returns: Pointer to decrypted data on success, NULL on failure
// 6425
pub type esp_ccmp_decrypt_t = ::core::option::Option<
    unsafe extern "C" fn(
        tk: *const u8,
        ieee80211_hdr: *const u8,
        data: *const u8,
        data_len: usize,
        decrypted_len: *mut usize,
        espnow_pkt: bool,
    ) -> *mut u8,
>;

/// @brief Encrypt data callback function using CCMP (Counter Mode CBC-MAC Protocol OR
///         Counter Mode Cipher Block Chaining Message Authentication
///         Code Protocol) which is used in IEEE 802.11i RSN standard.
/// @param tk 128-bit Temporal Key for obtained during 4-way handshake
/// @param frame Pointer to IEEE802.11 frame including header
/// @param len Length of the frame including header
/// @param hdrlen Length of the header
/// @param pn Packet Number counter
/// @param keyid Key ID to be mentioned in CCMP Vector
/// @param encrypted_len Length of the encrypted frame including header
// 6436
pub type esp_ccmp_encrypt_t = ::core::option::Option<
    unsafe extern "C" fn(
        tk: *const u8,
        frame: *mut u8,
        len: usize,
        hdrlen: usize,
        pn: *mut u8,
        keyid: crate::c_types::c_int,
        encrypted_len: *mut usize,
    ) -> *mut u8,
>;

/// @brief One-Key GMAC hash callback function with AES for MIC computation
///
/// @param key key for the hash operation
/// @param keylen key length
/// @param iv initialization vector
/// @param iv_len initialization vector length
/// @param aad aad
/// @param aad_len aad length
/// @param mic Buffer for MIC (128 bits, i.e., 16 bytes)
/// Returns: 0 on success, -1 on failure"]
// 6448
pub type esp_aes_gmac_t = ::core::option::Option<
    unsafe extern "C" fn(
        key: *const u8,
        keylen: usize,
        iv: *const u8,
        iv_len: usize,
        aad: *const u8,
        aad_len: usize,
        mic: *mut u8,
    ) -> crate::c_types::c_int,
>;

/// @brief SHA256 hash callback function for data vector
///
/// @param num_elem Number of elements in the data vector
/// @param addr Pointers to the data areas
/// @param len Lengths of the data blocks
/// @param buf Buffer for the hash
/// Returns: 0 on success, -1 on failure
// 6460
pub type esp_sha256_vector_t = ::core::option::Option<
    unsafe extern "C" fn(
        num_elem: usize,
        addr: *mut *const u8,
        len: *const usize,
        buf: *mut u8,
    ) -> crate::c_types::c_int,
>;

/// @brief CRC32 value callback function in little endian.
/// @param crc Initial CRC value (result of last calculation or 0 for the first time)
/// @param buf Data buffer that used to calculate the CRC value
/// @param len Length of the data buffer
/// @return CRC32 value
// 6468
pub type esp_crc32_le_t =
    ::core::option::Option<unsafe extern "C" fn(crc: u32, buf: *const u8, len: u32) -> u32>;

/// @brief The crypto callback function structure used by esp_wifi.
///         The structure can be set as software crypto or the crypto optimized by device's
///         hardware."]
#[repr(C)]
#[derive(Copy, Clone)]
// 6474
pub struct wpa_crypto_funcs_t {
    /// < The crypto callback function structure size
    pub size: u32,
    /// < The crypto callback function structure version
    pub version: u32,
    /// < The AES wrap callback function used by esp_wifi
    pub aes_wrap: esp_aes_wrap_t,
    /// < The AES unwrap callback function used by esp_wifi
    pub aes_unwrap: esp_aes_unwrap_t,
    /// < The SHA256 callback function used by esp_wifi
    pub hmac_sha256_vector: esp_hmac_sha256_vector_t,
    /// < The SHA256 PRF callback function used by esp_wifi
    pub sha256_prf: esp_sha256_prf_t,
    /// < HMAC-MD5 callback function over data buffer (RFC 2104)
    pub hmac_md5: esp_hmac_md5_t,
    /// < HMAC-MD5 callback function over data vector (RFC 2104)
    pub hamc_md5_vector: esp_hmac_md5_vector_t,
    /// < HMAC-SHA1 callback function over data buffer (RFC 2104)
    pub hmac_sha1: esp_hmac_sha1_t,
    /// < HMAC-SHA1 callback function over data vector (RFC 2104)
    pub hmac_sha1_vector: esp_hmac_sha1_vector_t,
    /// < SHA1-based Pseudo-Random Function (PRF) (IEEE 802.11i, 8.5.1.1) callback function
    pub sha1_prf: esp_sha1_prf_t,
    /// < SHA-1 hash callback function for data vector
    pub sha1_vector: esp_sha1_vector_t,
    /// < SHA1-based key derivation function (PBKDF2) callback function for IEEE 802.11i
    pub pbkdf2_sha1: esp_pbkdf2_sha1_t,
    /// < XOR RC4 stream callback function to given data with skip-stream-start
    pub rc4_skip: esp_rc4_skip_t,
    /// < MD5 hash callback function for data vector
    pub md5_vector: esp_md5_vector_t,
    /// < Encrypt one AES block callback function
    pub aes_encrypt: esp_aes_encrypt_t,
    /// < Initialize AES callback function for encryption
    pub aes_encrypt_init: esp_aes_encrypt_init_t,
    /// < Deinitialize AES encryption callback function
    pub aes_encrypt_deinit: esp_aes_encrypt_deinit_t,
    /// < Decrypt one AES block callback function
    pub aes_decrypt: esp_aes_decrypt_t,
    /// < Initialize AES callback function for decryption
    pub aes_decrypt_init: esp_aes_decrypt_init_t,
    /// "< Deinitialize AES decryption callback function
    pub aes_decrypt_deinit: esp_aes_decrypt_deinit_t,
    /// < The AES 128 encrypt callback function used by esp_wifi
    pub aes_128_encrypt: esp_aes_128_encrypt_t,
    /// < The AES 128 decrypt callback function used by esp_wifi
    pub aes_128_decrypt: esp_aes_128_decrypt_t,
    /// < One-Key CBC MAC (OMAC1) hash with AES-128 callback function for MIC computation
    pub omac1_aes_128: esp_omac1_aes_128_t,
    /// < Decrypt data callback function using CCMP
    pub ccmp_decrypt: esp_ccmp_decrypt_t,
    /// < Encrypt data callback function using CCMP
    pub ccmp_encrypt: esp_ccmp_encrypt_t,
    /// < One-Key GMAC hash callback function with AES for MIC computation
    pub aes_gmac: esp_aes_gmac_t,
    /// < SHA256 hash callback function for data vector
    pub sha256_vector: esp_sha256_vector_t,
    /// < CRC32 value callback function in little endian
    pub crc32: esp_crc32_le_t,
}

/// @brief WiFi stack configuration parameters passed to esp_wifi_init call.
#[repr(C)]
#[derive(Copy, Clone)]
// 6602
pub struct wifi_init_config_t {
    /// < WiFi OS functions
    pub osi_funcs: *mut wifi_osi_funcs_t,
    /// < WiFi station crypto functions when connect
    pub wpa_crypto_funcs: wpa_crypto_funcs_t,
    /// < WiFi static RX buffer number
    pub static_rx_buf_num: crate::c_types::c_int,
    /// < WiFi dynamic RX buffer number
    pub dynamic_rx_buf_num: crate::c_types::c_int,
    /// < WiFi TX buffer type
    pub tx_buf_type: crate::c_types::c_int,
    /// < WiFi static TX buffer number
    pub static_tx_buf_num: crate::c_types::c_int,
    /// < WiFi dynamic TX buffer number
    pub dynamic_tx_buf_num: crate::c_types::c_int,
    /// < WiFi RX MGMT buffer type
    pub rx_mgmt_buf_type: crate::c_types::c_int,
    /// < WiFi RX MGMT buffer number
    pub rx_mgmt_buf_num: crate::c_types::c_int,
    /// < WiFi TX cache buffer number
    pub cache_tx_buf_num: crate::c_types::c_int,
    /// < WiFi channel state information enable flag
    pub csi_enable: crate::c_types::c_int,
    /// < WiFi AMPDU RX feature enable flag
    pub ampdu_rx_enable: crate::c_types::c_int,
    /// < WiFi AMPDU TX feature enable flag
    pub ampdu_tx_enable: crate::c_types::c_int,
    /// < WiFi AMSDU TX feature enable flag
    pub amsdu_tx_enable: crate::c_types::c_int,
    /// < WiFi NVS flash enable flag
    pub nvs_enable: crate::c_types::c_int,
    /// "< Nano option for printf/scan family enable flag
    pub nano_enable: crate::c_types::c_int,
    /// < WiFi Block Ack RX window size
    pub rx_ba_win: crate::c_types::c_int,
    /// "< WiFi Task Core ID
    pub wifi_task_core_id: crate::c_types::c_int,
    /// < WiFi softAP maximum length of the beacon
    pub beacon_max_len: crate::c_types::c_int,
    /// < WiFi management short buffer number, the minimum value is 6, the maximum value is 32
    pub mgmt_sbuf_num: crate::c_types::c_int,
    /// < Enables additional WiFi features and capabilities
    pub feature_caps: u64,
    /// < WiFi Power Management for station at disconnected status
    pub sta_disconnected_pm: bool,
    /// < Maximum encrypt number of peers supported by espnow
    pub espnow_max_encrypt_num: crate::c_types::c_int,
    /// < WiFi TX HE TB QUEUE number for STA HE TB PPDU transmission
    pub tx_hetb_queue_num: crate::c_types::c_int,
    /// < enable dump sigb field
    pub dump_hesigb_enable: bool,
    /// < WiFi init magic number, it should be the last field
    pub magic: crate::c_types::c_int,
}

// 6656
unsafe extern "C" {
    /// @addtogroup WPA_APIs
    /// @{
    pub static g_wifi_default_wpa_crypto_funcs: wpa_crypto_funcs_t;
}

// 6660
unsafe extern "C" {
    pub static mut g_wifi_osi_funcs: wifi_osi_funcs_t;
}

// 7159
pub const wifi_log_level_t_WIFI_LOG_NONE: wifi_log_level_t = 0;
pub const wifi_log_level_t_WIFI_LOG_ERROR: wifi_log_level_t = 1;
pub const wifi_log_level_t_WIFI_LOG_WARNING: wifi_log_level_t = 2;
pub const wifi_log_level_t_WIFI_LOG_INFO: wifi_log_level_t = 3;
pub const wifi_log_level_t_WIFI_LOG_DEBUG: wifi_log_level_t = 4;
pub const wifi_log_level_t_WIFI_LOG_VERBOSE: wifi_log_level_t = 5;
/// @brief WiFi log level\n
// 7166
pub type wifi_log_level_t = crate::c_types::c_uint;

// 7173
unsafe extern "C" {
    /// @brief Initialize Wi-Fi Driver
    ///     Alloc resource for WiFi driver, such as WiFi control structure, RX/TX buffer,
    ///     WiFi NVS structure among others.
    ///
    /// For the most part, you need not call this function directly. It gets called
    /// from esp_wifi_init().
    ///
    /// This function may be called, if you only need to initialize the Wi-Fi driver
    /// without having to use the network stack on top.
    ///
    /// @param  config provide WiFi init configuration
    ///
    /// @return
    ///     - ESP_OK: succeed
    ///     - ESP_ERR_NO_MEM: out of memory
    ///     - others: refer to error code esp_err.h
    pub fn esp_wifi_init_internal(config: *const wifi_init_config_t) -> esp_err_t;
}

// 7308
unsafe extern "C" {
    /// @brief     Set current WiFi log level
    ///
    /// @param     level   Log level.
    ///
    /// @return
    ///     - ESP_OK: succeed
    ///     - ESP_FAIL: level is invalid
    pub fn esp_wifi_internal_set_log_level(level: wifi_log_level_t) -> esp_err_t;
}

#[repr(C)]
#[derive(Copy, Clone)]
// 7451
pub struct wifi_osi_funcs_t {
    pub _version: i32,
    pub _env_is_chip: ::core::option::Option<unsafe extern "C" fn() -> bool>,
    pub _set_intr: ::core::option::Option<
        unsafe extern "C" fn(cpu_no: i32, intr_source: u32, intr_num: u32, intr_prio: i32),
    >,
    pub _clear_intr: ::core::option::Option<unsafe extern "C" fn(intr_source: u32, intr_num: u32)>,
    pub _set_isr: ::core::option::Option<
        unsafe extern "C" fn(
            n: i32,
            f: *mut crate::c_types::c_void,
            arg: *mut crate::c_types::c_void,
        ),
    >,
    pub _ints_on: ::core::option::Option<unsafe extern "C" fn(mask: u32)>,
    pub _ints_off: ::core::option::Option<unsafe extern "C" fn(mask: u32)>,
    pub _is_from_isr: ::core::option::Option<unsafe extern "C" fn() -> bool>,
    pub _spin_lock_create:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _spin_lock_delete:
        ::core::option::Option<unsafe extern "C" fn(lock: *mut crate::c_types::c_void)>,
    pub _wifi_int_disable: ::core::option::Option<
        unsafe extern "C" fn(wifi_int_mux: *mut crate::c_types::c_void) -> u32,
    >,
    pub _wifi_int_restore: ::core::option::Option<
        unsafe extern "C" fn(wifi_int_mux: *mut crate::c_types::c_void, tmp: u32),
    >,
    pub _task_yield_from_isr: ::core::option::Option<unsafe extern "C" fn()>,
    pub _semphr_create: ::core::option::Option<
        unsafe extern "C" fn(max: u32, init: u32) -> *mut crate::c_types::c_void,
    >,
    pub _semphr_delete:
        ::core::option::Option<unsafe extern "C" fn(semphr: *mut crate::c_types::c_void)>,
    pub _semphr_take: ::core::option::Option<
        unsafe extern "C" fn(semphr: *mut crate::c_types::c_void, block_time_tick: u32) -> i32,
    >,
    pub _semphr_give:
        ::core::option::Option<unsafe extern "C" fn(semphr: *mut crate::c_types::c_void) -> i32>,
    pub _wifi_thread_semphr_get:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _mutex_create:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _recursive_mutex_create:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _mutex_delete:
        ::core::option::Option<unsafe extern "C" fn(mutex: *mut crate::c_types::c_void)>,
    pub _mutex_lock:
        ::core::option::Option<unsafe extern "C" fn(mutex: *mut crate::c_types::c_void) -> i32>,
    pub _mutex_unlock:
        ::core::option::Option<unsafe extern "C" fn(mutex: *mut crate::c_types::c_void) -> i32>,
    pub _queue_create: ::core::option::Option<
        unsafe extern "C" fn(queue_len: u32, item_size: u32) -> *mut crate::c_types::c_void,
    >,
    pub _queue_delete:
        ::core::option::Option<unsafe extern "C" fn(queue: *mut crate::c_types::c_void)>,
    pub _queue_send: ::core::option::Option<
        unsafe extern "C" fn(
            queue: *mut crate::c_types::c_void,
            item: *mut crate::c_types::c_void,
            block_time_tick: u32,
        ) -> i32,
    >,
    pub _queue_send_from_isr: ::core::option::Option<
        unsafe extern "C" fn(
            queue: *mut crate::c_types::c_void,
            item: *mut crate::c_types::c_void,
            hptw: *mut crate::c_types::c_void,
        ) -> i32,
    >,
    pub _queue_send_to_back: ::core::option::Option<
        unsafe extern "C" fn(
            queue: *mut crate::c_types::c_void,
            item: *mut crate::c_types::c_void,
            block_time_tick: u32,
        ) -> i32,
    >,
    pub _queue_send_to_front: ::core::option::Option<
        unsafe extern "C" fn(
            queue: *mut crate::c_types::c_void,
            item: *mut crate::c_types::c_void,
            block_time_tick: u32,
        ) -> i32,
    >,
    pub _queue_recv: ::core::option::Option<
        unsafe extern "C" fn(
            queue: *mut crate::c_types::c_void,
            item: *mut crate::c_types::c_void,
            block_time_tick: u32,
        ) -> i32,
    >,
    pub _queue_msg_waiting:
        ::core::option::Option<unsafe extern "C" fn(queue: *mut crate::c_types::c_void) -> u32>,
    pub _event_group_create:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _event_group_delete:
        ::core::option::Option<unsafe extern "C" fn(event: *mut crate::c_types::c_void)>,
    pub _event_group_set_bits: ::core::option::Option<
        unsafe extern "C" fn(event: *mut crate::c_types::c_void, bits: u32) -> u32,
    >,
    pub _event_group_clear_bits: ::core::option::Option<
        unsafe extern "C" fn(event: *mut crate::c_types::c_void, bits: u32) -> u32,
    >,
    pub _event_group_wait_bits: ::core::option::Option<
        unsafe extern "C" fn(
            event: *mut crate::c_types::c_void,
            bits_to_wait_for: u32,
            clear_on_exit: crate::c_types::c_int,
            wait_for_all_bits: crate::c_types::c_int,
            block_time_tick: u32,
        ) -> u32,
    >,
    pub _task_create_pinned_to_core: ::core::option::Option<
        unsafe extern "C" fn(
            task_func: *mut crate::c_types::c_void,
            name: *const crate::c_types::c_char,
            stack_depth: u32,
            param: *mut crate::c_types::c_void,
            prio: u32,
            task_handle: *mut crate::c_types::c_void,
            core_id: u32,
        ) -> i32,
    >,
    pub _task_create: ::core::option::Option<
        unsafe extern "C" fn(
            task_func: *mut crate::c_types::c_void,
            name: *const crate::c_types::c_char,
            stack_depth: u32,
            param: *mut crate::c_types::c_void,
            prio: u32,
            task_handle: *mut crate::c_types::c_void,
        ) -> i32,
    >,
    pub _task_delete:
        ::core::option::Option<unsafe extern "C" fn(task_handle: *mut crate::c_types::c_void)>,
    pub _task_delay: ::core::option::Option<unsafe extern "C" fn(tick: u32)>,
    pub _task_ms_to_tick: ::core::option::Option<unsafe extern "C" fn(ms: u32) -> i32>,
    pub _task_get_current_task:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _task_get_max_priority: ::core::option::Option<unsafe extern "C" fn() -> i32>,
    pub _malloc:
        ::core::option::Option<unsafe extern "C" fn(size: usize) -> *mut crate::c_types::c_void>,
    pub _free: ::core::option::Option<unsafe extern "C" fn(p: *mut crate::c_types::c_void)>,
    pub _event_post: ::core::option::Option<
        unsafe extern "C" fn(
            event_base: *const crate::c_types::c_char,
            event_id: i32,
            event_data: *mut crate::c_types::c_void,
            event_data_size: usize,
            ticks_to_wait: u32,
        ) -> i32,
    >,
    pub _get_free_heap_size: ::core::option::Option<unsafe extern "C" fn() -> u32>,
    pub _rand: ::core::option::Option<unsafe extern "C" fn() -> u32>,
    pub _dport_access_stall_other_cpu_start_wrap: ::core::option::Option<unsafe extern "C" fn()>,
    pub _dport_access_stall_other_cpu_end_wrap: ::core::option::Option<unsafe extern "C" fn()>,
    pub _wifi_apb80m_request: ::core::option::Option<unsafe extern "C" fn()>,
    pub _wifi_apb80m_release: ::core::option::Option<unsafe extern "C" fn()>,
    pub _phy_disable: ::core::option::Option<unsafe extern "C" fn()>,
    pub _phy_enable: ::core::option::Option<unsafe extern "C" fn()>,
    pub _phy_update_country_info: ::core::option::Option<
        unsafe extern "C" fn(country: *const crate::c_types::c_char) -> crate::c_types::c_int,
    >,
    pub _read_mac: ::core::option::Option<
        unsafe extern "C" fn(mac: *mut u8, type_: crate::c_types::c_uint) -> crate::c_types::c_int,
    >,
    pub _timer_arm: ::core::option::Option<
        unsafe extern "C" fn(timer: *mut crate::c_types::c_void, tmout: u32, repeat: bool),
    >,
    pub _timer_disarm:
        ::core::option::Option<unsafe extern "C" fn(timer: *mut crate::c_types::c_void)>,
    pub _timer_done:
        ::core::option::Option<unsafe extern "C" fn(ptimer: *mut crate::c_types::c_void)>,
    pub _timer_setfn: ::core::option::Option<
        unsafe extern "C" fn(
            ptimer: *mut crate::c_types::c_void,
            pfunction: *mut crate::c_types::c_void,
            parg: *mut crate::c_types::c_void,
        ),
    >,
    pub _timer_arm_us: ::core::option::Option<
        unsafe extern "C" fn(ptimer: *mut crate::c_types::c_void, us: u32, repeat: bool),
    >,
    pub _wifi_reset_mac: ::core::option::Option<unsafe extern "C" fn()>,
    pub _wifi_clock_enable: ::core::option::Option<unsafe extern "C" fn()>,
    pub _wifi_clock_disable: ::core::option::Option<unsafe extern "C" fn()>,
    pub _wifi_rtc_enable_iso: ::core::option::Option<unsafe extern "C" fn()>,
    pub _wifi_rtc_disable_iso: ::core::option::Option<unsafe extern "C" fn()>,
    pub _esp_timer_get_time: ::core::option::Option<unsafe extern "C" fn() -> i64>,
    pub _nvs_set_i8: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            value: i8,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_get_i8: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            out_value: *mut i8,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_set_u8: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            value: u8,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_get_u8: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            out_value: *mut u8,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_set_u16: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            value: u16,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_get_u16: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            out_value: *mut u16,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_open: ::core::option::Option<
        unsafe extern "C" fn(
            name: *const crate::c_types::c_char,
            open_mode: crate::c_types::c_uint,
            out_handle: *mut u32,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_close: ::core::option::Option<unsafe extern "C" fn(handle: u32)>,
    pub _nvs_commit:
        ::core::option::Option<unsafe extern "C" fn(handle: u32) -> crate::c_types::c_int>,
    pub _nvs_set_blob: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            value: *const crate::c_types::c_void,
            length: usize,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_get_blob: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
            out_value: *mut crate::c_types::c_void,
            length: *mut usize,
        ) -> crate::c_types::c_int,
    >,
    pub _nvs_erase_key: ::core::option::Option<
        unsafe extern "C" fn(
            handle: u32,
            key: *const crate::c_types::c_char,
        ) -> crate::c_types::c_int,
    >,
    pub _get_random: ::core::option::Option<
        unsafe extern "C" fn(buf: *mut u8, len: usize) -> crate::c_types::c_int,
    >,
    pub _get_time: ::core::option::Option<
        unsafe extern "C" fn(t: *mut crate::c_types::c_void) -> crate::c_types::c_int,
    >,
    pub _random: ::core::option::Option<unsafe extern "C" fn() -> crate::c_types::c_ulong>,
    pub _slowclk_cal_get: ::core::option::Option<unsafe extern "C" fn() -> u32>,
    pub _log_write: ::core::option::Option<
        unsafe extern "C" fn(
            level: crate::c_types::c_uint,
            tag: *const crate::c_types::c_char,
            format: *const crate::c_types::c_char,
            ...
        ),
    >,
    pub _log_writev: ::core::option::Option<
        unsafe extern "C" fn(
            level: crate::c_types::c_uint,
            tag: *const crate::c_types::c_char,
            format: *const crate::c_types::c_char,
            args: va_list,
        ),
    >,
    pub _log_timestamp: ::core::option::Option<unsafe extern "C" fn() -> u32>,
    pub _malloc_internal:
        ::core::option::Option<unsafe extern "C" fn(size: usize) -> *mut crate::c_types::c_void>,
    pub _realloc_internal: ::core::option::Option<
        unsafe extern "C" fn(
            ptr: *mut crate::c_types::c_void,
            size: usize,
        ) -> *mut crate::c_types::c_void,
    >,
    pub _calloc_internal: ::core::option::Option<
        unsafe extern "C" fn(n: usize, size: usize) -> *mut crate::c_types::c_void,
    >,
    pub _zalloc_internal:
        ::core::option::Option<unsafe extern "C" fn(size: usize) -> *mut crate::c_types::c_void>,
    pub _wifi_malloc:
        ::core::option::Option<unsafe extern "C" fn(size: usize) -> *mut crate::c_types::c_void>,
    pub _wifi_realloc: ::core::option::Option<
        unsafe extern "C" fn(
            ptr: *mut crate::c_types::c_void,
            size: usize,
        ) -> *mut crate::c_types::c_void,
    >,
    pub _wifi_calloc: ::core::option::Option<
        unsafe extern "C" fn(n: usize, size: usize) -> *mut crate::c_types::c_void,
    >,
    pub _wifi_zalloc:
        ::core::option::Option<unsafe extern "C" fn(size: usize) -> *mut crate::c_types::c_void>,
    pub _wifi_create_queue: ::core::option::Option<
        unsafe extern "C" fn(
            queue_len: crate::c_types::c_int,
            item_size: crate::c_types::c_int,
        ) -> *mut crate::c_types::c_void,
    >,
    pub _wifi_delete_queue:
        ::core::option::Option<unsafe extern "C" fn(queue: *mut crate::c_types::c_void)>,
    pub _coex_init: ::core::option::Option<unsafe extern "C" fn() -> crate::c_types::c_int>,
    pub _coex_deinit: ::core::option::Option<unsafe extern "C" fn()>,
    pub _coex_enable: ::core::option::Option<unsafe extern "C" fn() -> crate::c_types::c_int>,
    pub _coex_disable: ::core::option::Option<unsafe extern "C" fn()>,
    pub _coex_status_get: ::core::option::Option<unsafe extern "C" fn() -> u32>,
    pub _coex_condition_set:
        ::core::option::Option<unsafe extern "C" fn(type_: u32, dissatisfy: bool)>,
    pub _coex_wifi_request: ::core::option::Option<
        unsafe extern "C" fn(event: u32, latency: u32, duration: u32) -> crate::c_types::c_int,
    >,
    pub _coex_wifi_release:
        ::core::option::Option<unsafe extern "C" fn(event: u32) -> crate::c_types::c_int>,
    pub _coex_wifi_channel_set: ::core::option::Option<
        unsafe extern "C" fn(primary: u8, secondary: u8) -> crate::c_types::c_int,
    >,
    pub _coex_event_duration_get: ::core::option::Option<
        unsafe extern "C" fn(event: u32, duration: *mut u32) -> crate::c_types::c_int,
    >,
    pub _coex_pti_get: ::core::option::Option<
        unsafe extern "C" fn(event: u32, pti: *mut u8) -> crate::c_types::c_int,
    >,
    pub _coex_schm_status_bit_clear:
        ::core::option::Option<unsafe extern "C" fn(type_: u32, status: u32)>,
    pub _coex_schm_status_bit_set:
        ::core::option::Option<unsafe extern "C" fn(type_: u32, status: u32)>,
    pub _coex_schm_interval_set:
        ::core::option::Option<unsafe extern "C" fn(interval: u32) -> crate::c_types::c_int>,
    pub _coex_schm_interval_get: ::core::option::Option<unsafe extern "C" fn() -> u32>,
    pub _coex_schm_curr_period_get: ::core::option::Option<unsafe extern "C" fn() -> u8>,
    pub _coex_schm_curr_phase_get:
        ::core::option::Option<unsafe extern "C" fn() -> *mut crate::c_types::c_void>,
    pub _coex_schm_process_restart:
        ::core::option::Option<unsafe extern "C" fn() -> crate::c_types::c_int>,
    pub _coex_schm_register_cb: ::core::option::Option<
        unsafe extern "C" fn(
            arg1: crate::c_types::c_int,
            cb: ::core::option::Option<
                unsafe extern "C" fn(arg1: crate::c_types::c_int) -> crate::c_types::c_int,
            >,
        ) -> crate::c_types::c_int,
    >,
    pub _coex_register_start_cb: ::core::option::Option<
        unsafe extern "C" fn(
            cb: ::core::option::Option<unsafe extern "C" fn() -> crate::c_types::c_int>,
        ) -> crate::c_types::c_int,
    >,
    pub _coex_schm_flexible_period_set:
        ::core::option::Option<unsafe extern "C" fn(arg1: u8) -> crate::c_types::c_int>,
    pub _coex_schm_flexible_period_get: ::core::option::Option<unsafe extern "C" fn() -> u8>,
    pub _magic: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
// 9117
pub struct timer_adpt {
    pub _address: u8,
}

// 9120
pub type __builtin_va_list = *mut crate::c_types::c_void;

// 9123
//unsafe impl Sync for wifi_osi_funcs_t {}
