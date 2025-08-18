#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Tunable parameters for the WiFi driver
#[allow(unused)] // currently there are no ble tunables
// 5
pub(crate) struct EspWifiConfig {
    /// Size of the RX queue in frames
    pub(crate) rx_queue_size: usize,
    /// Size of the TX queue in frames
    pub(crate) tx_queue_size: usize,
    /// WiFi static RX buffer number
    pub(crate) static_rx_buf_num: usize,
    /// WiFi dynamic RX buffer number
    pub(crate) dynamic_rx_buf_num: usize,
    /// WiFi static TX buffer number
    pub(crate) static_tx_buf_num: usize,
    /// WiFi dynamic TX buffer number
    pub(crate) dynamic_tx_buf_num: usize,
    /// WiFi AMPDU RX feature enable flag
    pub(crate) ampdu_rx_enable: bool,
    /// WiFi AMPDU TX feature enable flag
    pub(crate) ampdu_tx_enable: bool,
    /// WiFi AMSDU TX feature enable flag
    pub(crate) amsdu_tx_enable: bool,
    /// WiFi Block Ack RX window size
    pub(crate) rx_ba_win: usize,
    /// See [smoltcp's documentation]
    pub(crate) max_burst_size: usize,
    /// "Country code
    pub(crate) country_code: &'static str,
    /// If not 0: Operating Class table number
    pub(crate) country_code_operating_class: u8,
    /// MTU, see [smoltcp's documentation]
    pub(crate) mtu: usize,
    /// Tick rate of the internal task scheduler in hertz
    pub(crate) tick_rate_hz: u32,
    /// Interval for station to listen to beacon from AP
    /// The unit of listen interval is one beacon interval.
    /// For example, if beacon interval is 100 ms and listen interval is 3,
    /// the interval for station to listen to beacon is 300 ms
    pub(crate) listen_interval: u16,
    /// For Station, If the station does not receive a beacon frame
    /// from the connected SoftAP during the  inactive time, disconnect from SoftAP.
    /// Default 6s. Range 6-30
    pub(crate) beacon_timeout: u16,
    /// For SoftAP, If the SoftAP doesn't receive any data from the connected STA
    /// during inactive time, the SoftAP will force deauth the STA. Default is 300s
    pub(crate) ap_beacon_timeout: u16,
    /// Number of connection retries station will do before moving to next AP.
    /// scan_method should be set as WIFI_ALL_CHANNEL_SCAN to use this config.
    /// Note: Enabling this may cause connection time to increase incase best AP
    /// doesn't behave properly. Defaults to 1
    pub(crate) failure_retry_cnt: u8,
    /// 0 = WIFI_FAST_SCAN, 1 = WIFI_ALL_CHANNEL_SCAN, defaults to 0
    pub(crate) scan_method: u32,
}

//pub const ESP_WIFI_CONFIG_RX_QUEUE_SIZE: usize = 5;
pub const ESP_WIFI_CONFIG_RX_QUEUE_SIZE: usize = 8;
pub const ESP_WIFI_CONFIG_TX_QUEUE_SIZE: usize = 3;
pub const ESP_WIFI_CONFIG_STATIC_RX_BUF_NUM: usize = 10;
pub const ESP_WIFI_CONFIG_DYNAMIC_RX_BUF_NUM: usize = 32;
pub const ESP_WIFI_CONFIG_STATIC_TX_BUF_NUM: usize = 0;
pub const ESP_WIFI_CONFIG_DYNAMIC_TX_BUF_NUM: usize = 32;
pub const ESP_WIFI_CONFIG_AMPDU_RX_ENABLE: bool = true;
pub const ESP_WIFI_CONFIG_AMPDU_TX_ENABLE: bool = true;
pub const ESP_WIFI_CONFIG_AMSDU_TX_ENABLE: bool = false;
pub const ESP_WIFI_CONFIG_RX_BA_WIN: usize = 6;
pub const ESP_WIFI_CONFIG_MAX_BURST_SIZE: usize = 1;
pub const ESP_WIFI_CONFIG_COUNTRY_CODE: &'static str = "HU";
pub const ESP_WIFI_CONFIG_COUNTRY_CODE_OPERATING_CLASS: u8 = 0;
pub const ESP_WIFI_CONFIG_MTU: usize = 1492;
pub const ESP_WIFI_CONFIG_TICK_RATE_HZ: u32 = 100;
pub const ESP_WIFI_CONFIG_LISTEN_INTERVAL: u16 = 3;
pub const ESP_WIFI_CONFIG_BEACON_TIMEOUT: u16 = 6;
pub const ESP_WIFI_CONFIG_AP_BEACON_TIMEOUT: u16 = 300;
pub const ESP_WIFI_CONFIG_FAILURE_RETRY_CNT: u8 = 1;
pub const ESP_WIFI_CONFIG_SCAN_METHOD: u32 = 0;
