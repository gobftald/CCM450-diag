//! WiFi

// 3
pub mod event;
pub mod internal;
pub(crate) mod os_adapter;
pub(crate) mod state;

// 7
use alloc::{collections::vec_deque::VecDeque, string::String};
use core::{marker::PhantomData, ptr::addr_of, task::Poll};

// 19
use enumset::{EnumSet, EnumSetType};

// 21
use esp_hal::{asynch::AtomicWaker, system::Cpu};
use esp_sync::NonReentrantMutex;

use esp_config::{esp_config_bool, esp_config_int, esp_config_str};

// 35
use num_derive::FromPrimitive;

// 37
pub(crate) use os_adapter::*;
use portable_atomic::{AtomicUsize, Ordering};

#[cfg(feature = "smoltcp")]
// don't use smoltcp from here, but via embassy_net
// 42
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
// 43
pub use state::*;

// 45
use crate::{
    Controller, common_adapter::read_mac, esp_wifi_result, hal::ram, wifi::private::PacketBuffer,
};

// 50
//const MTU: usize = crate::CONFIG.mtu;
const MTU: usize = esp_config_int!(usize, "ESP_WIFI_CONFIG_MTU");

// 105
use crate::binary::{
    c_types,
    c_types::c_uint,
    include::{
        self, __BindgenBitfieldUnit, WIFI_INIT_CONFIG_MAGIC, esp_err_t,
        esp_interface_t_ESP_IF_WIFI_AP, esp_interface_t_ESP_IF_WIFI_STA, esp_supplicant_init,
        esp_wifi_connect, esp_wifi_disconnect, esp_wifi_get_mode, esp_wifi_init_internal,
        esp_wifi_internal_free_rx_buffer, esp_wifi_internal_reg_rxcb, esp_wifi_internal_tx,
        esp_wifi_set_config, esp_wifi_set_country, esp_wifi_set_mode, esp_wifi_set_tx_done_cb,
        esp_wifi_start, g_wifi_default_wpa_crypto_funcs, wifi_ap_config_t, wifi_auth_mode_t,
        wifi_cipher_type_t_WIFI_CIPHER_TYPE_CCMP, wifi_config_t,
        wifi_country_policy_t_WIFI_COUNTRY_POLICY_MANUAL, wifi_country_t, wifi_init_config_t,
        wifi_interface_t, wifi_interface_t_WIFI_IF_AP, wifi_interface_t_WIFI_IF_STA, wifi_mode_t,
        wifi_mode_t_WIFI_MODE_AP, wifi_mode_t_WIFI_MODE_APSTA, wifi_mode_t_WIFI_MODE_NULL,
        wifi_mode_t_WIFI_MODE_STA, wifi_pmf_config_t, wifi_scan_threshold_t,
        wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL, wifi_sta_config_t,
    },
};

/// Supported Wi-Fi authentication methods.
// 158
#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
pub enum AuthMethod {
    /// No authentication (open network).
    None,

    /// Wired Equivalent Privacy (WEP) authentication.
    Wep,

    /// Wi-Fi Protected Access (WPA) authentication.
    Wpa,

    /// Wi-Fi Protected Access 2 (WPA2) Personal authentication (default).
    #[default]
    Wpa2Personal,

    /// WPA/WPA2 Personal authentication (supports both).
    WpaWpa2Personal,

    /// WPA2 Enterprise authentication.
    Wpa2Enterprise,

    /// WPA3 Personal authentication.
    Wpa3Personal,

    /// WPA2/WPA3 Personal authentication (supports both).
    Wpa2Wpa3Personal,

    /// WLAN Authentication and Privacy Infrastructure (WAPI).
    WapiPersonal,
}

/// Supported Wi-Fi protocols.
// 193
#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
pub enum Protocol {
    /// 802.11b protocol.
    P802D11B,

    /// 802.11b/g protocol.
    P802D11BG,

    /// 802.11b/g/n protocol (default).
    #[default]
    P802D11BGN,

    /// 802.11b/g/n long-range (LR) protocol.
    P802D11BGNLR,

    /// 802.11 long-range (LR) protocol.
    P802D11LR,

    /// 802.11b/g/n/ax protocol.
    P802D11BGNAX,
}

/// Configuration for a Wi-Fi access point.
// 326
#[derive(Clone, PartialEq, Eq)]
pub struct AccessPointConfig {
    /// The SSID of the access point.
    pub ssid: String,

    /// Whether the SSID is hidden or visible.
    pub ssid_hidden: bool,

    /// The channel the access point will operate on.
    pub channel: u8,

    /// The secondary channel configuration.
    pub secondary_channel: Option<u8>,

    /// The set of protocols supported by the access point.
    pub protocols: EnumSet<Protocol>,

    /// The authentication method to be used by the access point.
    pub auth_method: AuthMethod,

    /// The password for securing the access point (if applicable).
    pub password: String,

    /// The maximum number of connections allowed on the access point.
    pub max_connections: u16,

    /// Dtim period of the access point (Range: 1 ~ 10).
    pub dtim_period: u8,

    /// Time to force deauth the STA if the SoftAP doesn't receive any data.
    pub beacon_timeout: u16,
}

// 362
impl AccessPointConfig {
    fn validate(&self) -> Result<(), WifiError> {
        if self.ssid.len() > 32 {
            return Err(WifiError::InvalidArguments);
        }

        if self.password.len() > 64 {
            return Err(WifiError::InvalidArguments);
        }

        if !(1..=10).contains(&self.dtim_period) {
            return Err(WifiError::InvalidArguments);
        }

        Ok(())
    }
}

// 380
impl Default for AccessPointConfig {
    fn default() -> Self {
        Self {
            ssid: String::new(),
            ssid_hidden: false,
            channel: 1,
            secondary_channel: None,
            protocols: (Protocol::P802D11B | Protocol::P802D11BG | Protocol::P802D11BGN),
            auth_method: AuthMethod::None,
            password: String::new(),
            max_connections: 255,
            dtim_period: 2,
            beacon_timeout: 300,
        }
    }
}

// 397
impl core::fmt::Debug for AccessPointConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AccessPointConfiguration")
            .field("ssid", &self.ssid)
            .field("ssid_hidden", &self.ssid_hidden)
            .field("channel", &self.channel)
            .field("secondary_channel", &self.secondary_channel)
            .field("protocols", &self.protocols)
            .field("auth_method", &self.auth_method)
            .field("password", &"**REDACTED**")
            .field("max_connections", &self.max_connections)
            .field("dtim_period", &self.dtim_period)
            .field("beacon_timeout", &self.beacon_timeout)
            .finish()
    }
}

// 414
#[cfg(feature = "defmt")]
impl defmt::Format for AccessPointConfig {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Default)]
        pub struct ProtocolSet(EnumSet<Protocol>);

        #[cfg(feature = "defmt")]
        impl defmt::Format for ProtocolSet {
            fn format(&self, fmt: defmt::Formatter<'_>) {
                for (i, p) in self.0.into_iter().enumerate() {
                    if i > 0 {
                        defmt::write!(fmt, " ");
                    }
                    defmt::write!(fmt, "{}", p);
                }
            }
        }

        let protocol_set = ProtocolSet(self.protocols);

        defmt::write!(
            fmt,
            "AccessPointConfiguration {{\
            ssid: {}, \
            ssid_hidden: {}, \
            channel: {}, \
            secondary_channel: {}, \
            protocols: {}, \
            auth_method: {}, \
            password: **REDACTED**, \
            max_connections: {}, \
            dtim_period: {}, \
            beacon_timeout: {} \
            }}",
            self.ssid.as_str(),
            self.ssid_hidden,
            self.channel,
            self.secondary_channel,
            protocol_set,
            self.auth_method,
            self.max_connections,
            self.dtim_period,
            self.beacon_timeout
        );
    }
}

/// Wi-Fi scan method.
// 445
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ScanMethod {
    /// Fast scan.
    Fast,

    /// Scan all channels.
    AllChannels,
}

/// Client configuration for a Wi-Fi connection.
// 458
#[derive(Clone, PartialEq, Eq)]
pub struct ClientConfig {
    /// The SSID of the Wi-Fi network.
    pub ssid: String,

    /// The BSSID (MAC address) of the client.
    pub bssid: Option<[u8; 6]>,

    // pub protocol: Protocol,
    /// The authentication method for the Wi-Fi connection.
    pub auth_method: AuthMethod,

    /// The password for the Wi-Fi connection.
    pub password: String,

    /// The Wi-Fi channel to connect to.
    pub channel: Option<u8>,

    /// Scan method.
    scan_method: ScanMethod,

    /// Interval for station to listen to beacon from AP.
    ///
    /// The unit of listen interval is one beacon interval.
    /// For example, if beacon interval is 100 ms and listen interval is 3,
    /// the interval for station to listen to beacon is 300 ms
    listen_interval: u16,

    /// Time to disconnect from AP if no data is received.
    ///
    /// Must be between 6 and 31.
    beacon_timeout: u16,

    /// Number of connection retries station will do before moving to next AP.
    ///
    /// `scan_method` should be set as [`ScanMethod::AllChannels`] to use this config.
    ///
    /// Note: Enabling this may cause connection time to increase in case the best AP
    /// doesn't behave properly.
    failure_retry_cnt: u8,
}

// 509
impl ClientConfig {
    fn validate(&self) -> Result<(), WifiError> {
        if self.ssid.len() > 32 {
            return Err(WifiError::InvalidArguments);
        }

        if self.password.len() > 64 {
            return Err(WifiError::InvalidArguments);
        }

        if !(6..=31).contains(&self.beacon_timeout) {
            return Err(WifiError::InvalidArguments);
        }

        Ok(())
    }
}

// 527
impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            ssid: String::new(),
            bssid: None,
            auth_method: AuthMethod::Wpa2Personal,
            password: String::new(),
            channel: None,
            scan_method: ScanMethod::Fast,
            listen_interval: 3,
            beacon_timeout: 6,
            failure_retry_cnt: 1,
        }
    }
}

// 544
impl core::fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClientConfiguration")
            .field("ssid", &self.ssid)
            .field("bssid", &self.bssid)
            .field("auth_method", &self.auth_method)
            .field("password", &"**REDACTED**")
            .field("channel", &self.channel)
            .field("listen_interval", &self.listen_interval)
            .field("beacon_timeout", &self.beacon_timeout)
            .field("failure_retry_cnt", &self.failure_retry_cnt)
            .finish()
    }
}

// 561
#[cfg(feature = "defmt")]
impl defmt::Format for ClientConfig {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(
            fmt,
            "ClientConfiguration {{\
            ssid: {}, \
            bssid: {:?}, \
            auth_method: {:?}, \
            password: **REDACTED**, \
            channel: {:?}, \
            listen_interval: {}, \
            beacon_timeout: {}, \
            failure_retry_cnt: {}, \
            }}",
            self.ssid.as_str(),
            self.bssid,
            self.auth_method,
            self.channel,
            self.listen_interval,
            self.beacon_timeout,
            self.failure_retry_cnt,
        )
    }
}

/// Introduces Wi-Fi configuration options.
// 879
#[derive(EnumSetType, Debug, PartialOrd)]
//#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Capability {
    /// The device operates as a client, connecting to an existing network.
    Client,

    /// The device operates as an access point, allowing other devices to
    /// connect to it.
    AccessPoint,

    /// The device can operate in both client and access point modes
    /// simultaneously.
    Mixed,
}

#[cfg(feature = "defmt")]
impl defmt::Format for Capability {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Capability::Client => defmt::write!(f, "Client"),
            Capability::AccessPoint => defmt::write!(f, "AccessPoint"),
            Capability::Mixed => defmt::write!(f, "Mixed"),
        }
    }
}

/// Configuration of Wi-Fi operation mode.
// 897
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModeConfig {
    /// No configuration (default).
    #[default]
    None,

    /// Client-only configuration.
    Client(ClientConfig),

    /// Access point-only configuration.
    AccessPoint(AccessPointConfig),

    /// Simultaneous client and access point configuration.
    Mixed(ClientConfig, AccessPointConfig),
    /*
    /// EAP client configuration for enterprise Wi-Fi.
    #[cfg_attr(feature = "serde", serde(skip))]
    EapClient(EapClientConfiguration),
    */
}

// 922
impl ModeConfig {
    fn validate(&self) -> Result<(), WifiError> {
        match self {
            ModeConfig::None => Ok(()),
            ModeConfig::Client(client_configuration) => client_configuration.validate(),
            ModeConfig::AccessPoint(access_point_configuration) => {
                access_point_configuration.validate()
            }
            ModeConfig::Mixed(client_configuration, access_point_configuration) => {
                client_configuration.validate()?;
                access_point_configuration.validate()
            } /*
              ModeConfig::EapClient(eap_client_configuration) => {
                  eap_client_configuration.validate()
              }
              */
        }
    }
}

// 940
trait AuthMethodExt {
    fn to_raw(&self) -> wifi_auth_mode_t;
}

// 945
impl AuthMethodExt for AuthMethod {
    fn to_raw(&self) -> wifi_auth_mode_t {
        match self {
            AuthMethod::None => include::wifi_auth_mode_t_WIFI_AUTH_OPEN,
            AuthMethod::Wep => include::wifi_auth_mode_t_WIFI_AUTH_WEP,
            AuthMethod::Wpa => include::wifi_auth_mode_t_WIFI_AUTH_WPA_PSK,
            AuthMethod::Wpa2Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK,
            AuthMethod::WpaWpa2Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK,
            AuthMethod::Wpa2Enterprise => include::wifi_auth_mode_t_WIFI_AUTH_WPA2_ENTERPRISE,
            AuthMethod::Wpa3Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA3_PSK,
            AuthMethod::Wpa2Wpa3Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA2_WPA3_PSK,
            AuthMethod::WapiPersonal => include::wifi_auth_mode_t_WIFI_AUTH_WAPI_PSK,
        }
    }
}

/// Wifi Mode (Sta and/or Ap)
// 977
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WifiMode {
    /// Station mode.
    Sta,
    /// Access Point mode.
    Ap,
    /// Both Station and Access Point modes.
    ApSta,
}

// 990
impl WifiMode {
    pub(crate) fn current() -> Result<Self, WifiError> {
        let mut mode = wifi_mode_t_WIFI_MODE_NULL;
        esp_wifi_result!(unsafe { esp_wifi_get_mode(&mut mode) })?;

        Self::try_from(mode)
    }

    /// Returns true if this mode works as a client
    // 999
    pub fn is_sta(&self) -> bool {
        match self {
            Self::Sta | Self::ApSta => true,
            Self::Ap => false,
        }
    }

    /// Returns true if this mode works as an access point
    // 1007
    pub fn is_ap(&self) -> bool {
        match self {
            Self::Sta => false,
            Self::Ap | Self::ApSta => true,
        }
    }
}

// 1015
impl TryFrom<&ModeConfig> for WifiMode {
    type Error = WifiError;

    /// Converts a `wifi_mode_t` C-type into a `WifiMode`.
    fn try_from(config: &ModeConfig) -> Result<Self, Self::Error> {
        #[allow(non_upper_case_globals)]
        match config {
            ModeConfig::Client(_) => Ok(Self::Sta),
            ModeConfig::AccessPoint(_) => Ok(Self::Ap),
            ModeConfig::Mixed(_, _) => Ok(Self::ApSta),
            _ => Err(WifiError::UnknownWifiMode),
        }
    }
}

// 1034
impl TryFrom<wifi_mode_t> for WifiMode {
    type Error = WifiError;

    /// Converts a `wifi_mode_t` C-type into a `WifiMode`.
    fn try_from(value: wifi_mode_t) -> Result<Self, Self::Error> {
        #[allow(non_upper_case_globals)]
        match value {
            include::wifi_mode_t_WIFI_MODE_STA => Ok(Self::Sta),
            include::wifi_mode_t_WIFI_MODE_AP => Ok(Self::Ap),
            include::wifi_mode_t_WIFI_MODE_APSTA => Ok(Self::ApSta),
            _ => Err(WifiError::UnknownWifiMode),
        }
    }
}

// 1248
static RX_QUEUE_SIZE: AtomicUsize = AtomicUsize::new(0);
static TX_QUEUE_SIZE: AtomicUsize = AtomicUsize::new(0);

// 1251
pub(crate) static DATA_QUEUE_RX_AP: NonReentrantMutex<VecDeque<PacketBuffer>> =
    NonReentrantMutex::new(VecDeque::new());
pub(crate) static DATA_QUEUE_RX_STA: NonReentrantMutex<VecDeque<PacketBuffer>> =
    NonReentrantMutex::new(VecDeque::new());

/// Common errors.
// 1258
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum WifiError {
    /// Wi-Fi module is not initialized or not initialized for `Wi-Fi`
    /// operations.
    NotInitialized,

    /// Internal Wi-Fi error.
    InternalError(InternalWifiError),

    /// The device disconnected from the network or failed to connect to it.
    Disconnected,

    /// Unknown Wi-Fi mode (not Sta/Ap/ApSta).
    UnknownWifiMode,

    /// Unsupported operation or mode.
    Unsupported,

    /// Passed arguments are invalid.
    InvalidArguments,
}

/// Events generated by the WiFi driver.
// 1299
#[repr(i32)]
#[derive(Debug, FromPrimitive, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WifiEvent {
    /// Wi-Fi is ready for operation.
    WifiReady = 0,
    /// Scan operation has completed.
    ScanDone,
    /// Station mode started.
    StaStart,
    /// Station mode stopped.
    StaStop,
    /// Station connected to a network.
    StaConnected,
    /// Station disconnected from a network.
    StaDisconnected,
    /// Station authentication mode changed.
    StaAuthmodeChange,

    /// Station WPS succeeds in enrollee mode.
    StaWpsErSuccess,
    /// Station WPS fails in enrollee mode.
    StaWpsErFailed,
    /// Station WPS timeout in enrollee mode.
    StaWpsErTimeout,
    /// Station WPS pin code in enrollee mode.
    StaWpsErPin,
    /// Station WPS overlap in enrollee mode.
    StaWpsErPbcOverlap,

    /// Soft-AP start.
    ApStart,
    /// Soft-AP stop.
    ApStop,

    /// A station connected to Soft-AP.
    ApStaconnected,
    /// A station disconnected from Soft-AP.
    ApStadisconnected,
    /// Received probe request packet in Soft-AP interface.
    ApProbereqrecved,

    /// Received report of FTM procedure.
    FtmReport,

    /// AP's RSSI crossed configured threshold.
    StaBssRssiLow,
    /// Status indication of Action Tx operation.
    ActionTxStatus,
    /// Remain-on-Channel operation complete.
    RocDone,

    /// Station beacon timeout.
    StaBeaconTimeout,

    /// Connectionless module wake interval has started.
    ConnectionlessModuleWakeIntervalStart,

    /// Soft-AP WPS succeeded in registrar mode.
    ApWpsRgSuccess,
    /// Soft-AP WPS failed in registrar mode.
    ApWpsRgFailed,
    /// Soft-AP WPS timed out in registrar mode.
    ApWpsRgTimeout,
    /// Soft-AP WPS pin code in registrar mode.
    ApWpsRgPin,
    /// Soft-AP WPS overlap in registrar mode.
    ApWpsRgPbcOverlap,

    /// iTWT setup.
    ItwtSetup,
    /// iTWT teardown.
    ItwtTeardown,
    /// iTWT probe.
    ItwtProbe,
    /// iTWT suspended.
    ItwtSuspend,
    /// TWT wakeup event.
    TwtWakeup,
    /// bTWT setup.
    BtwtSetup,
    /// bTWT teardown.
    BtwtTeardown,

    /// NAN (Neighbor Awareness Networking) discovery has started.
    NanStarted,
    /// NAN discovery has stopped.
    NanStopped,
    /// NAN service discovery match found.
    NanSvcMatch,
    /// Replied to a NAN peer with service discovery match.
    NanReplied,
    /// Received a follow-up message in NAN.
    NanReceive,
    /// Received NDP (Neighbor Discovery Protocol) request from a NAN peer.
    NdpIndication,
    /// NDP confirm indication.
    NdpConfirm,
    /// NAN datapath terminated indication.
    NdpTerminated,
    /// Wi-Fi home channel change, doesn't occur when scanning.
    HomeChannelChange,

    /// Received Neighbor Report response.
    StaNeighborRep,
}

/// Error originating from the underlying drivers
// 1407
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InternalWifiError {
    /// Out of memory
    EspErrNoMem = 0x101,

    /// Invalid argument
    EspErrInvalidArg = 0x102,

    /// WiFi driver was not installed by esp_wifi_init
    EspErrWifiNotInit = 0x3001,

    /// WiFi driver was not started by esp_wifi_start
    EspErrWifiNotStarted = 0x3002,

    /// WiFi driver was not stopped by esp_wifi_stop
    EspErrWifiNotStopped = 0x3003,

    /// WiFi interface error
    EspErrWifiIf = 0x3004,

    /// WiFi mode error
    EspErrWifiMode = 0x3005,

    /// WiFi internal state error
    EspErrWifiState = 0x3006,

    /// WiFi internal control block of station or soft-AP error
    EspErrWifiConn = 0x3007,

    /// WiFi internal NVS module error
    EspErrWifiNvs = 0x3008,

    /// MAC address is invalid
    EspErrWifiMac = 0x3009,

    /// SSID is invalid
    EspErrWifiSsid = 0x300A,

    /// Password is invalid
    EspErrWifiPassword = 0x300B,

    /// Timeout error
    EspErrWifiTimeout = 0x300C,

    /// WiFi is in sleep state(RF closed) and wakeup fail
    EspErrWifiWakeFail = 0x300D,

    /// The caller would block
    EspErrWifiWouldBlock = 0x300E,

    /// Station still in disconnect status
    EspErrWifiNotConnect = 0x300F,

    /// Failed to post the event to WiFi task
    EspErrWifiPost = 0x3012,

    /// Invalid WiFi state when init/deinit is called
    EspErrWifiInitState = 0x3013,

    /// Returned when WiFi is stopping
    EspErrWifiStopState = 0x3014,

    /// The WiFi connection is not associated
    EspErrWifiNotAssoc = 0x3015,

    /// The WiFi TX is disallowed
    EspErrWifiTxDisallow = 0x3016,
}

/// Get the AP MAC address
// 1480
pub fn ap_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 1);
    }
}

/// Get the STA MAC address
// 1489
pub fn sta_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 0);
    }
}

// 1508
pub(crate) fn wifi_init(_wifi: crate::hal::peripherals::WIFI<'_>) -> Result<(), WifiError> {
    unsafe {
        esp_wifi_result!(esp_wifi_init_internal(addr_of!(internal::G_CONFIG)))?;
        esp_wifi_result!(esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_NULL))?;

        esp_wifi_result!(esp_supplicant_init())?;

        esp_wifi_result!(esp_wifi_set_tx_done_cb(Some(esp_wifi_tx_done_cb)))?;

        esp_wifi_result!(esp_wifi_internal_reg_rxcb(
            esp_interface_t_ESP_IF_WIFI_STA,
            Some(recv_cb_sta)
        ))?;

        // until we support APSTA we just register the same callback for AP and STA
        esp_wifi_result!(esp_wifi_internal_reg_rxcb(
            esp_interface_t_ESP_IF_WIFI_AP,
            Some(recv_cb_ap)
        ))?;

        /*
        crate::flags::WIFI.store(true, Ordering::SeqCst);
        */

        Ok(())
    }
}

// 1576
unsafe extern "C" fn recv_cb_sta(
    buffer: *mut c_types::c_void,
    len: u16,
    eb: *mut c_types::c_void,
) -> esp_err_t {
    let packet = PacketBuffer { buffer, len, eb };
    // We must handle the result outside of the lock because
    // EspWifiPacketBuffer::drop must not be called in a critical section.
    // Dropping an EspWifiPacketBuffer will call `esp_wifi_internal_free_rx_buffer`
    // which will try to lock an internal mutex. If the mutex is already taken,
    // the function will try to trigger a context switch, which will fail if we
    // are in an interrupt-free context.
    match DATA_QUEUE_RX_STA.with(|queue| {
        if queue.len() < RX_QUEUE_SIZE.load(Ordering::Relaxed) {
            queue.push_back(packet);
            Ok(())
        } else {
            Err(packet)
        }
    }) {
        Ok(()) => {
            embassy::STA_RECEIVE_WAKER.wake();
            include::ESP_OK as esp_err_t
        }
        _ => {
            debug!("RX QUEUE FULL");
            include::ESP_ERR_NO_MEM as esp_err_t
        }
    }
}

// 1607
unsafe extern "C" fn recv_cb_ap(
    buffer: *mut c_types::c_void,
    len: u16,
    eb: *mut c_types::c_void,
) -> esp_err_t {
    let packet = PacketBuffer { buffer, len, eb };
    // We must handle the result outside of the critical section because
    // EspWifiPacketBuffer::drop must not be called in a critical section.
    // Dropping an EspWifiPacketBuffer will call `esp_wifi_internal_free_rx_buffer`
    // which will try to lock an internal mutex. If the mutex is already taken,
    // the function will try to trigger a context switch, which will fail if we
    // are in an interrupt-free context.
    match DATA_QUEUE_RX_AP.with(|queue| {
        if queue.len() < RX_QUEUE_SIZE.load(Ordering::Relaxed) {
            queue.push_back(packet);
            Ok(())
        } else {
            Err(packet)
        }
    }) {
        Ok(()) => {
            embassy::AP_RECEIVE_WAKER.wake();
            include::ESP_OK as esp_err_t
        }
        _ => {
            debug!("RX QUEUE FULL");
            include::ESP_ERR_NO_MEM as esp_err_t
        }
    }
}

// 1638
pub(crate) static WIFI_TX_INFLIGHT: AtomicUsize = AtomicUsize::new(0);

// 1414
fn decrement_inflight_counter() {
    unwrap!(
        WIFI_TX_INFLIGHT.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
            Some(x.saturating_sub(1))
        })
    );
}

// 1648
#[ram]
unsafe extern "C" fn esp_wifi_tx_done_cb(
    _ifidx: u8,
    _data: *mut u8,
    _data_len: *mut u16,
    _tx_status: bool,
) {
    //trace!("esp_wifi_tx_done_cb");
    info!("esp_wifi_tx_done_cb");

    decrement_inflight_counter();

    embassy::TRANSMIT_WAKER.wake();
}

// 1813
mod private {
    use super::*;

    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    /// Take care not to drop this while in a critical section.
    ///
    /// Dropping an EspWifiPacketBuffer will call
    /// `esp_wifi_internal_free_rx_buffer` which will try to lock an
    /// internal mutex. If the mutex is already taken, the function will try
    /// to trigger a context switch, which will fail if we are in a critical
    /// section.
    pub struct PacketBuffer {
        pub(crate) buffer: *mut c_types::c_void,
        pub(crate) len: u16,
        pub(crate) eb: *mut c_types::c_void,
    }

    // 1831
    unsafe impl Send for PacketBuffer {}

    // 1833
    impl Drop for PacketBuffer {
        fn drop(&mut self) {
            trace!("Dropping EspWifiPacketBuffer, freeing memory");
            unsafe { esp_wifi_internal_free_rx_buffer(self.eb) };
        }
    }

    // 1632
    impl PacketBuffer {
        pub fn as_slice_mut(&mut self) -> &mut [u8] {
            unsafe { core::slice::from_raw_parts_mut(self.buffer as *mut u8, self.len as usize) }
        }
    }
}

/// Provides methods for retrieving the Wi-Fi mode and MAC address.
// 1848
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WifiDeviceMode {
    Sta,
    Ap,
}

// 1646
impl WifiDeviceMode {
    fn mac_address(&self) -> [u8; 6] {
        match self {
            WifiDeviceMode::Sta => {
                let mut mac = [0; 6];
                sta_mac(&mut mac);
                mac
            }
            WifiDeviceMode::Ap => {
                let mut mac = [0; 6];
                ap_mac(&mut mac);
                mac
            }
        }
    }

    // 1864
    fn data_queue_rx(&self) -> &'static NonReentrantMutex<VecDeque<PacketBuffer>> {
        match self {
            WifiDeviceMode::Sta => &DATA_QUEUE_RX_STA,
            WifiDeviceMode::Ap => &DATA_QUEUE_RX_AP,
        }
    }

    // 1871
    fn can_send(&self) -> bool {
        WIFI_TX_INFLIGHT.load(Ordering::SeqCst) < TX_QUEUE_SIZE.load(Ordering::Relaxed)
    }

    // 1875
    fn increase_in_flight_counter(&self) {
        WIFI_TX_INFLIGHT.fetch_add(1, Ordering::SeqCst);
    }

    // 1879
    fn tx_token(&self) -> Option<WifiTxToken> {
        if !self.can_send() {
            crate::preempt::yield_task();
        }

        if self.can_send() {
            Some(WifiTxToken { mode: *self })
        } else {
            None
        }
    }

    // 1892
    fn rx_token(&self) -> Option<(WifiRxToken, WifiTxToken)> {
        let is_empty = self.data_queue_rx().with(|q| q.is_empty());
        if is_empty || !self.can_send() {
            crate::preempt::yield_task();
        }

        let is_empty = is_empty && self.data_queue_rx().with(|q| q.is_empty());

        if !is_empty {
            self.tx_token().map(|tx| (WifiRxToken { mode: *self }, tx))
        } else {
            None
        }
    }

    // 1908
    fn interface(&self) -> wifi_interface_t {
        match self {
            WifiDeviceMode::Sta => wifi_interface_t_WIFI_IF_STA,
            WifiDeviceMode::Ap => wifi_interface_t_WIFI_IF_AP,
        }
    }

    // 1915
    fn register_transmit_waker(&self, cx: &mut core::task::Context<'_>) {
        embassy::TRANSMIT_WAKER.register(cx.waker())
    }

    // 1919
    fn register_receive_waker(&self, cx: &mut core::task::Context<'_>) {
        match self {
            WifiDeviceMode::Sta => embassy::STA_RECEIVE_WAKER.register(cx.waker()),
            WifiDeviceMode::Ap => embassy::AP_RECEIVE_WAKER.register(cx.waker()),
        }
    }

    // 1926
    fn register_link_state_waker(&self, cx: &mut core::task::Context<'_>) {
        match self {
            WifiDeviceMode::Sta => embassy::STA_LINK_STATE_WAKER.register(cx.waker()),
            WifiDeviceMode::Ap => embassy::AP_LINK_STATE_WAKER.register(cx.waker()),
        }
    }

    // 1933
    fn link_state(&self) -> embassy_net_driver::LinkState {
        match self {
            WifiDeviceMode::Sta => {
                if matches!(sta_state(), WifiState::StaConnected) {
                    embassy_net_driver::LinkState::Up
                } else {
                    embassy_net_driver::LinkState::Down
                }
            }
            WifiDeviceMode::Ap => {
                if matches!(ap_state(), WifiState::ApStarted) {
                    embassy_net_driver::LinkState::Up
                } else {
                    embassy_net_driver::LinkState::Down
                }
            }
        }
    }
}

/// A wifi device implementing smoltcp's Device trait.
// 1954
pub struct WifiDevice<'d> {
    _phantom: PhantomData<&'d ()>,
    mode: WifiDeviceMode,
}

// 1959
impl WifiDevice<'_> {
    /// Retrieves the MAC address of the Wi-Fi device.
    pub fn mac_address(&self) -> [u8; 6] {
        self.mode.mac_address()
    }
}

// 2320
#[derive(Debug)]
pub struct WifiRxToken {
    mode: WifiDeviceMode,
}

// 22325
impl WifiRxToken {
    /// Consumes the RX token and applies the callback function to the received
    /// data buffer.
    pub fn consume_token<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut data = self.mode.data_queue_rx().with(|queue| {
            unwrap!(
                queue.pop_front(),
                "unreachable: transmit()/receive() ensures there is a packet to process"
            )
        });

        // We handle the received data outside of the lock because
        // EspWifiPacketBuffer::drop must not be called in a critical section.
        // Dropping an EspWifiPacketBuffer will call `esp_wifi_internal_free_rx_buffer`
        // which will try to lock an internal mutex. If the mutex is already
        // taken, the function will try to trigger a context switch, which will
        // fail if we are in an interrupt-free context.
        let buffer = data.as_slice_mut();
        dump_packet_info(buffer, self.mode, '<');

        f(buffer)
    }
}

// 2364
#[derive(Debug)]
pub struct WifiTxToken {
    mode: WifiDeviceMode,
}

// 2369
impl WifiTxToken {
    /// Consumes the TX token and applies the callback function to the received
    /// data buffer.
    pub fn consume_token<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        self.mode.increase_in_flight_counter();

        // (safety): creation of multiple WiFi devices with the same mode is impossible
        // in safe Rust, therefore only smoltcp _or_ embassy-net can be used at
        // one time
        static mut BUFFER: [u8; MTU] = [0u8; MTU];

        let buffer = unsafe { &mut BUFFER[..len] };

        let res = f(buffer);

        esp_wifi_send_data(self.mode, buffer);

        res
    }
}

// FIXME data here has to be &mut because of `esp_wifi_internal_tx` signature,
// requiring a *mut ptr to the buffer Casting const to mut is instant UB, even
// though in reality `esp_wifi_internal_tx` copies the buffer into its own
// memory and does not modify
// 2408
pub(crate) fn esp_wifi_send_data(mode: WifiDeviceMode, data: &mut [u8]) {
    trace!("sending... {} bytes", data.len());
    dump_packet_info(data, mode, '>');

    let len = data.len() as u16;
    let ptr = data.as_mut_ptr().cast();

    let res = unsafe { esp_wifi_internal_tx(mode.interface(), ptr, len) };

    if res != 0 {
        warn!("esp_wifi_internal_tx {}", res);
        decrement_inflight_counter();
    } else {
        trace!("esp_wifi_internal_tx ok");
    }
}

// 2425
fn dump_packet_info(_buffer: &mut [u8], _mode: WifiDeviceMode, _direction: char) {
    #[cfg(dump_packets)]
    #[cfg(feature = "defmt")]
    {
        //info!("@WIFIFRAME {:?}", _buffer);

        let mut addr1: [u8; 4] = [0; 4];
        let mut addr2: [u8; 4] = [0; 4];
        let port1: u16;
        let port2: u16;
        unsafe {
            if _direction == '>' {
                core::ptr::copy_nonoverlapping(&_buffer[26], &mut addr1 as *mut u8, 4);
                core::ptr::copy_nonoverlapping(&_buffer[30], &mut addr2 as *mut u8, 4);
                port1 = (_buffer[34] as u16) << 8 | (_buffer[35] as u16);
                port2 = (_buffer[36] as u16) << 8 | (_buffer[37] as u16);
            } else {
                core::ptr::copy_nonoverlapping(&_buffer[26], &mut addr2 as *mut u8, 4);
                core::ptr::copy_nonoverlapping(&_buffer[30], &mut addr1 as *mut u8, 4);
                port2 = (_buffer[34] as u16) << 8 | (_buffer[35] as u16);
                port1 = (_buffer[36] as u16) << 8 | (_buffer[37] as u16);
            }
        }
        match (_buffer[12] as u16) << 8 | (_buffer[13] as u16) {
            0x0800 => {
                match _buffer[23] {
                    0x01 => info!("@Icmp packet"),
                    0x11 => info!(
                        "@Udp {} {}.{}.{}.{} {} {}.{}.{}.{} {} {} {}",
                        _mode,
                        addr1[0],
                        addr1[1],
                        addr1[2],
                        addr1[3],
                        _direction,
                        addr2[0],
                        addr2[1],
                        addr2[2],
                        addr2[3],
                        port1,
                        _direction,
                        port2
                    ),
                    _ => {}
                }
                //info!("@Ipv4 packet arrived {:x}", _buffer);
            }
            0x0806 => info!("@Arp packet"),
            0x86DD => info!("@Ipv6 packet"),
            _ => info!("@WIFIFRAME {:x}", _buffer),
            //_ => {}
        }
    }
}

// 2434
#[macro_export]
macro_rules! esp_wifi_result {
    ($value:expr) => {{
        use num_traits::FromPrimitive;
        let result = $value;
        if result != esp_wifi_sys_esp32c3::include::ESP_OK as i32 {
            warn!("{} returned an error: {}", stringify!($value), result);
            Err(WifiError::InternalError(unwrap!(FromPrimitive::from_i32(
                result
            ))))
        } else {
            Ok::<(), WifiError>(())
        }
    }};
}

// 2449
pub(crate) mod embassy {
    use embassy_net_driver::{Capabilities, Driver, HardwareAddress, RxToken, TxToken};

    use super::*;

    // We can get away with a single tx waker because the transmit queue is shared
    // between interfaces.
    // 2457
    pub(crate) static TRANSMIT_WAKER: AtomicWaker = AtomicWaker::new();

    // 2459
    pub(crate) static AP_RECEIVE_WAKER: AtomicWaker = AtomicWaker::new();
    pub(crate) static AP_LINK_STATE_WAKER: AtomicWaker = AtomicWaker::new();

    // 2462
    pub(crate) static STA_RECEIVE_WAKER: AtomicWaker = AtomicWaker::new();
    pub(crate) static STA_LINK_STATE_WAKER: AtomicWaker = AtomicWaker::new();

    // 2465
    impl RxToken for WifiRxToken {
        fn consume<R, F>(self, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            self.consume_token(f)
        }
    }

    // 2474
    impl TxToken for WifiTxToken {
        fn consume<R, F>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            self.consume_token(len, f)
        }
    }

    // 2483
    impl Driver for WifiDevice<'_> {
        type RxToken<'a>
            = WifiRxToken
        where
            Self: 'a;
        type TxToken<'a>
            = WifiTxToken
        where
            Self: 'a;

        // 2493
        fn receive(
            &mut self,
            cx: &mut core::task::Context<'_>,
        ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
            self.mode.register_receive_waker(cx);
            self.mode.register_transmit_waker(cx);
            self.mode.rx_token()
        }

        // 2502
        fn transmit(&mut self, cx: &mut core::task::Context<'_>) -> Option<Self::TxToken<'_>> {
            self.mode.register_transmit_waker(cx);
            self.mode.tx_token()
        }

        // 2507
        fn link_state(
            &mut self,
            cx: &mut core::task::Context<'_>,
        ) -> embassy_net_driver::LinkState {
            self.mode.register_link_state_waker(cx);
            self.mode.link_state()
        }

        // 2515
        fn capabilities(&self) -> Capabilities {
            let mut caps = Capabilities::default();
            caps.max_transmission_unit = MTU;
            //caps.max_burst_size = if crate::CONFIG.max_burst_size == 0 {
            caps.max_burst_size = if esp_config_int!(usize, "ESP_WIFI_CONFIG_MAX_BURST_SIZE") == 0 {
                None
            } else {
                Some(esp_config_int!(usize, "ESP_WIFI_CONFIG_MAX_BURST_SIZE"))
            };
            caps
        }

        // 2530
        fn hardware_address(&self) -> HardwareAddress {
            HardwareAddress::Ethernet(self.mac_address())
        }
    }
}

/// Power saving mode settings for the modem.
// 2537
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerSaveMode {
    /// No power saving.
    #[default]
    None,
    /// Minimum power save mode. In this mode, station wakes up to receive beacon every DTIM
    /// period.
    Minimum,
    /// Maximum power save mode. In this mode, interval to receive beacons is determined by the
    /// `listen_interval` config option.
    Maximum,
}

// 2552
pub(crate) fn apply_power_saving(ps: PowerSaveMode) -> Result<(), WifiError> {
    esp_wifi_result!(unsafe {
        esp_wifi_sys_esp32c3::include::esp_wifi_set_ps(match ps {
            PowerSaveMode::None => esp_wifi_sys_esp32c3::include::wifi_ps_type_t_WIFI_PS_NONE,
            PowerSaveMode::Minimum => {
                esp_wifi_sys_esp32c3::include::wifi_ps_type_t_WIFI_PS_MIN_MODEM
            }
            PowerSaveMode::Maximum => {
                esp_wifi_sys_esp32c3::include::wifi_ps_type_t_WIFI_PS_MAX_MODEM
            }
        })
    })?;
    Ok(())
}

// 2579
#[non_exhaustive]
pub struct Interfaces<'d> {
    pub sta: WifiDevice<'d>,
    pub ap: WifiDevice<'d>,
    //#[cfg(feature = "esp-now")]
    //pub esp_now: crate::esp_now::EspNow<'d>,
    //#[cfg(feature = "sniffer")]
    //pub sniffer: Sniffer,
}

/// Wi-Fi operating class.
///
/// Refer to Annex E of IEEE Std 802.11-2020.
// 2598
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OperatingClass {
    /// The regulations under which the station/AP is operating encompass all environments for the
    /// current frequency band in the country.
    AllEnvironments,

    /// The regulations under which the station/AP is operating are for an outdoor environment only.
    Outdoors,

    /// The regulations under which the station/AP is operating are for an indoor environment only.
    Indoors,

    /// The station/AP is operating under a noncountry entity. The first two octets of the
    /// noncountry entity is two ASCII ‘XX’ characters.
    NonCountryEntity,

    /// Binary representation of the Operating Class table number currently in use. Refer to Annex E
    /// of IEEE Std 802.11-2020.
    Repr(u8),
}

// 2621
impl Default for OperatingClass {
    fn default() -> Self {
        OperatingClass::Repr(0) // TODO: is this valid?
    }
}

// 2627
impl OperatingClass {
    fn into_code(self) -> u8 {
        match self {
            OperatingClass::AllEnvironments => b' ',
            OperatingClass::Outdoors => b'O',
            OperatingClass::Indoors => b'I',
            OperatingClass::NonCountryEntity => b'X',
            OperatingClass::Repr(code) => code,
        }
    }
}

/// Country information.
///
/// Defaults to China (CN) with Operating Class "0".
///
/// To create a [`CountryInfo`] instance, use the `from` method first, then set additional
/// properties using the builder methods.
///
/// ## Example
///
/// ```rust,no_run
/// use esp_radio::wifi::{CountryInfo, OperatingClass};
///
/// let country_info = CountryInfo::from(*b"CN").operating_class(OperatingClass::Indoors);
/// ```
///
/// For more information, see the [Wi-Fi Country Code in the ESP-IDF documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/wifi.html#wi-fi-country-code).
// 2655
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CountryInfo {
    /// Country code.
    country: [u8; 2],

    /// Operating class.
    operating_class: OperatingClass,
}

// 2667
impl From<[u8; 2]> for CountryInfo {
    fn from(country: [u8; 2]) -> Self {
        Self {
            country,
            operating_class: OperatingClass::default(),
        }
    }
}

// 2676
impl CountryInfo {
    fn into_blob(self) -> wifi_country_t {
        wifi_country_t {
            cc: [
                self.country[0],
                self.country[1],
                self.operating_class.into_code(),
            ],
            // TODO: these may be valid defaults, but they should be configurable.
            schan: 1,
            nchan: 13,
            max_tx_power: 20,
            policy: wifi_country_policy_t_WIFI_COUNTRY_POLICY_MANUAL,
        }
    }
}

/// Wi-Fi configuration.
// 2693
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Power save mode.
    power_save_mode: PowerSaveMode,

    // country_code will be set up directly from ESP_WIFI_CONFIG_COUNTRY_CODE config
    /// Size of the RX queue in frames.
    rx_queue_size: usize,

    /// Size of the TX queue in frames.
    tx_queue_size: usize,

    /// Max number of WiFi static RX buffers.
    ///
    /// Each buffer takes approximately 1.6KB of RAM. The static rx buffers are allocated when
    /// esp_wifi_init is called, they are not freed until esp_wifi_deinit is called.
    ///
    /// WiFi hardware use these buffers to receive all 802.11 frames. A higher number may allow
    /// higher throughput but increases memory use. If [`Self::ampdu_rx_enable`] is enabled,
    /// this value is recommended to set equal or bigger than [`Self::rx_ba_win`] in order to
    /// achieve better throughput and compatibility with both stations and APs.
    static_rx_buf_num: u8,

    /// Max number of WiFi dynamic RX buffers
    ///
    /// Set the number of WiFi dynamic RX buffers, 0 means unlimited RX buffers will be allocated
    /// (provided sufficient free RAM). The size of each dynamic RX buffer depends on the size of
    /// the received data frame.
    ///
    /// For each received data frame, the WiFi driver makes a copy to an RX buffer and then
    /// delivers it to the high layer TCP/IP stack. The dynamic RX buffer is freed after the
    /// higher layer has successfully received the data frame.
    ///
    /// For some applications, WiFi data frames may be received faster than the application can
    /// process them. In these cases we may run out of memory if RX buffer number is unlimited
    /// (0).
    ///
    /// If a dynamic RX buffer limit is set, it should be at least the number of
    /// static RX buffers.
    dynamic_rx_buf_num: u16,

    /// Set the number of WiFi static TX buffers.
    ///
    /// Each buffer takes approximately 1.6KB of RAM.
    /// The static RX buffers are allocated when esp_wifi_init() is called, they are not released
    /// until esp_wifi_deinit() is called.
    ///
    /// For each transmitted data frame from the higher layer TCP/IP stack, the WiFi driver makes a
    /// copy of it in a TX buffer.
    ///
    /// For some applications especially UDP applications, the upper layer can deliver frames
    /// faster than WiFi layer can transmit. In these cases, we may run out of TX buffers.
    static_tx_buf_num: u8,

    /// Set the number of WiFi dynamic TX buffers.
    ///
    /// The size of each dynamic TX buffer is not fixed,
    /// it depends on the size of each transmitted data frame.
    ///
    /// For each transmitted frame from the higher layer TCP/IP stack, the WiFi driver makes a copy
    /// of it in a TX buffer.
    ///
    /// For some applications, especially UDP applications, the upper layer can deliver frames
    /// faster than WiFi layer can transmit. In these cases, we may run out of TX buffers.
    dynamic_tx_buf_num: u16,

    /// Select this option to enable AMPDU RX feature.
    ampdu_rx_enable: bool,

    /// Select this option to enable AMPDU TX feature.
    ampdu_tx_enable: bool,

    /// Select this option to enable AMSDU TX feature.
    amsdu_tx_enable: bool,

    /// Set the size of WiFi Block Ack RX window.
    ///
    /// Generally a bigger value means higher throughput and better compatibility but more memory.
    /// Most of time we should NOT change the default value unless special reason, e.g. test
    /// the maximum UDP RX throughput with iperf etc. For iperf test in shieldbox, the
    /// recommended value is 9~12.
    ///
    /// If PSRAM is used and WiFi memory is preferred to allocate in PSRAM first, the default and
    /// minimum value should be 16 to achieve better throughput and compatibility with both
    /// stations and APs.
    rx_ba_win: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            power_save_mode: PowerSaveMode::default(),

            //rx_queue_size: 5,
            rx_queue_size: esp_config_int!(usize, "ESP_WIFI_CONFIG_RX_QUEUE_SIZE"),
            //tx_queue_size: 3,
            tx_queue_size: esp_config_int!(usize, "ESP_WIFI_CONFIG_TX_QUEUE_SIZE"),

            //static_rx_buf_num: 10,
            static_rx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_STATIC_RX_BUF_NUM") as _,
            //dynamic_rx_buf_num: 32,
            dynamic_rx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_DYNAMIC_RX_BUF_NUM") as _,

            //static_tx_buf_num: 0,
            static_tx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_STATIC_TX_BUF_NUM") as _,
            //dynamic_tx_buf_num: 32,
            dynamic_tx_buf_num: esp_config_int!(usize, "ESP_WIFI_CONFIG_DYNAMIC_TX_BUF_NUM") as _,

            //ampdu_rx_enable: true,
            ampdu_rx_enable: esp_config_bool!("ESP_WIFI_CONFIG_AMPDU_RX_ENABLE"),
            //ampdu_tx_enable: true,
            ampdu_tx_enable: esp_config_bool!("ESP_WIFI_CONFIG_AMPDU_TX_ENABLE"),
            //amsdu_tx_enable: false,
            amsdu_tx_enable: esp_config_bool!("ESP_WIFI_CONFIG_AMSDU_TX_ENABLE"),

            //rx_ba_win: 6,
            rx_ba_win: esp_config_int!(usize, "ESP_WIFI_CONFIG_RX_BA_WIN") as _,
        }
    }
}

impl Config {
    fn validate(&self) {
        if self.rx_ba_win as u16 >= self.dynamic_rx_buf_num {
            warn!("RX BA window size should be less than the number of dynamic RX buffers.");
        }
        if self.rx_ba_win as u16 >= 2 * (self.static_rx_buf_num as u16) {
            warn!("RX BA window size should be less than twice the number of static RX buffers.");
        }
    }
}

/// Create a WiFi controller and it's associated interfaces.
///
/// Dropping the controller will deinitialize / stop WiFi.
///
/// Make sure to **not** call this function while interrupts are disabled.
// 2837
pub fn new<'d>(
    _inited: &'d Controller<'d>,
    device: crate::hal::peripherals::WIFI<'d>,
    config: Config,
) -> Result<(WifiController<'d>, Interfaces<'d>), WifiError> {
    if crate::is_interrupts_disabled() {
        return Err(WifiError::Unsupported);
    }

    config.validate();

    unsafe {
        internal::G_CONFIG = wifi_init_config_t {
            osi_funcs: (&raw const internal::__ESP_RADIO_G_WIFI_OSI_FUNCS).cast_mut(),

            wpa_crypto_funcs: g_wifi_default_wpa_crypto_funcs,
            static_rx_buf_num: config.static_rx_buf_num as _,
            dynamic_rx_buf_num: config.dynamic_rx_buf_num as _,
            tx_buf_type: esp_wifi_sys_esp32c3::include::CONFIG_ESP_WIFI_TX_BUFFER_TYPE as i32,
            static_tx_buf_num: config.static_tx_buf_num as _,
            dynamic_tx_buf_num: config.dynamic_tx_buf_num as _,
            rx_mgmt_buf_type: esp_wifi_sys_esp32c3::include::CONFIG_ESP_WIFI_DYNAMIC_RX_MGMT_BUF
                as i32,
            rx_mgmt_buf_num: esp_wifi_sys_esp32c3::include::CONFIG_ESP_WIFI_RX_MGMT_BUF_NUM_DEF
                as i32,
            cache_tx_buf_num: esp_wifi_sys_esp32c3::include::WIFI_CACHE_TX_BUFFER_NUM as i32,
            csi_enable: cfg!(feature = "csi") as i32,
            ampdu_rx_enable: config.ampdu_rx_enable as _,
            ampdu_tx_enable: config.ampdu_tx_enable as _,
            amsdu_tx_enable: config.amsdu_tx_enable as _,
            nvs_enable: 0,
            nano_enable: 0,
            rx_ba_win: config.rx_ba_win as _,
            wifi_task_core_id: Cpu::current() as _,
            beacon_max_len: esp_wifi_sys_esp32c3::include::WIFI_SOFTAP_BEACON_MAX_LEN as i32,
            mgmt_sbuf_num: esp_wifi_sys_esp32c3::include::WIFI_MGMT_SBUF_NUM as i32,
            feature_caps: internal::__ESP_RADIO_G_WIFI_FEATURE_CAPS,
            sta_disconnected_pm: false,
            espnow_max_encrypt_num:
                esp_wifi_sys_esp32c3::include::CONFIG_ESP_WIFI_ESPNOW_MAX_ENCRYPT_NUM as i32,

            tx_hetb_queue_num: 3,
            dump_hesigb_enable: false,

            magic: WIFI_INIT_CONFIG_MAGIC as i32,
        };

        RX_QUEUE_SIZE.store(config.rx_queue_size, Ordering::Relaxed);
        TX_QUEUE_SIZE.store(config.tx_queue_size, Ordering::Relaxed);
    }

    crate::wifi::wifi_init(device)?;

    unsafe {
        let country_code: [u8; 2] =
            <[u8; 2]>::try_from(esp_config_str!("ESP_WIFI_CONFIG_COUNTRY_CODE").as_bytes())
                .unwrap();
        let country_info = <CountryInfo>::try_from(country_code).unwrap();
        esp_wifi_result!(esp_wifi_set_country(&country_info.into_blob()))?;
    }

    let mut controller = WifiController {
        _phantom: Default::default(),
        beacon_timeout: 6,
        ap_beacon_timeout: 100,
    };

    controller.set_power_saving(config.power_save_mode)?;

    Ok((
        controller,
        Interfaces {
            sta: WifiDevice {
                _phantom: Default::default(),
                mode: WifiDeviceMode::Sta,
            },
            ap: WifiDevice {
                _phantom: Default::default(),
                mode: WifiDeviceMode::Ap,
            },
            //#[cfg(feature = "esp-now")]
            //esp_now: crate::esp_now::EspNow::new_internal(),
            //#[cfg(feature = "sniffer")]
            //sniffer: Sniffer::new(),
        },
    ))
}

// 2926
#[non_exhaustive]
pub struct WifiController<'d> {
    _phantom: PhantomData<&'d ()>,
    // Things we have to remember due to how esp-wifi works:
    beacon_timeout: u16,
    ap_beacon_timeout: u16,
}

// 2946
impl WifiController<'_> {
    /// Configures modem power saving.
    // 3020
    pub fn set_power_saving(&mut self, ps: PowerSaveMode) -> Result<(), WifiError> {
        apply_power_saving(ps)
    }

    /// Starts the Wi-Fi controller.
    ///
    /// This method is not blocking. To check if the controller has started, use the
    /// [`Self::is_started`] method.
    //3061
    pub(crate) fn start(&mut self) -> Result<(), WifiError> {
        unsafe {
            esp_wifi_result!(esp_wifi_start())?;

            let mode = WifiMode::current()?;

            // This is not an if-else because in AP-STA mode, both are true
            if mode.is_ap() {
                esp_wifi_result!(include::esp_wifi_set_inactive_time(
                    wifi_interface_t_WIFI_IF_AP,
                    self.ap_beacon_timeout
                ))?;
            }
            if mode.is_sta() {
                esp_wifi_result!(include::esp_wifi_set_inactive_time(
                    wifi_interface_t_WIFI_IF_STA,
                    self.beacon_timeout
                ))?;
            };
        }

        Ok(())
    }

    /// Get the supported capabilities of the controller.
    // 3139
    pub fn capabilities(&self) -> Result<EnumSet<crate::wifi::Capability>, WifiError> {
        //pub fn capabilities(&self) -> Result<EnumSet<crate::wifi::Capability>, InternalWifiError> {
        let caps =
            enumset::enum_set! { Capability::Client | Capability::AccessPoint | Capability::Mixed };
        //InternalWifiError::EspErrNoMem;
        Ok(caps)
        //Err(caps)
    }

    /// Set the configuration.
    ///
    /// This will set the mode accordingly.
    /// You need to use Wifi::connect() for connecting to an AP.
    ///
    /// Passing [Configuration::None] will disable both, AP and STA mode.
    ///
    /// If you don't intent to use WiFi anymore at all consider tearing down
    /// WiFi completely.
    // 3155
    pub fn set_config(&mut self, conf: &ModeConfig) -> Result<(), WifiError> {
        conf.validate()?;

        let mode = match conf {
            ModeConfig::None => wifi_mode_t_WIFI_MODE_NULL,
            ModeConfig::Client(_) => wifi_mode_t_WIFI_MODE_STA,
            ModeConfig::AccessPoint(_) => wifi_mode_t_WIFI_MODE_AP,
            ModeConfig::Mixed(_, _) => wifi_mode_t_WIFI_MODE_APSTA,
            //ModeConfig::EapClient(_) => wifi_mode_t_WIFI_MODE_STA,
        };

        esp_wifi_result!(unsafe { esp_wifi_set_mode(mode) })?;

        match conf {
            ModeConfig::None => Ok(()),
            ModeConfig::Client(config) => self.apply_sta_config(config),
            ModeConfig::AccessPoint(config) => self.apply_ap_config(config),
            ModeConfig::Mixed(sta_config, ap_config) => {
                self.apply_ap_config(ap_config)?;
                self.apply_sta_config(sta_config)
            }
        }
        .inspect_err(|_| {
            debug!(".inspect_err");
            // we/the driver might have applied a partial configuration
            // so we better disable AP/STA just in case the caller ignores the error we
            // return here - they will run into futher errors this way
            unsafe { esp_wifi_set_mode(wifi_mode_t_WIFI_MODE_NULL) };
        })?;

        Ok(())
    }

    // 3213
    fn connect_impl(&mut self) -> Result<(), WifiError> {
        esp_wifi_result!(unsafe { esp_wifi_connect() })
    }

    // 3218
    pub fn disconnect_impl(&mut self) -> Result<(), WifiError> {
        esp_wifi_result!(unsafe { esp_wifi_disconnect() })
    }

    // 3253
    fn mode(&self) -> Result<WifiMode, WifiError> {
        WifiMode::current()
    }

    /// Async version of [`crate::wifi::WifiController`]'s `start` method
    // 3279
    pub async fn start_async(&mut self) -> Result<(), WifiError> {
        let mut events = enumset::enum_set! {};

        let mode = self.mode()?;
        if mode.is_ap() {
            events |= WifiEvent::ApStart;
        }
        if mode.is_sta() {
            events |= WifiEvent::StaStart;
        }

        Self::clear_events(events);

        self.start()?;

        self.wait_for_all_events(events, false).await;

        Ok(())
    }

    /// Async version of [`crate::wifi::WifiController`]'s `connect` method
    // 3328
    pub async fn connect_async(&mut self) -> Result<(), WifiError> {
        Self::clear_events(WifiEvent::StaConnected | WifiEvent::StaDisconnected);

        let err = crate::wifi::WifiController::connect_impl(self).err();

        if MultiWifiEventFuture::new(WifiEvent::StaConnected | WifiEvent::StaDisconnected)
            .await
            .contains(WifiEvent::StaDisconnected)
        {
            Err(err.unwrap_or(WifiError::Disconnected))
        } else {
            Ok(())
        }
    }

    // 3361
    fn clear_events(events: impl Into<EnumSet<WifiEvent>>) {
        WIFI_EVENTS.with(|evts| evts.remove_all(events.into()));
    }

    /// Wait for one [`WifiEvent`].
    // 3366
    pub async fn wait_for_event(&mut self, event: WifiEvent) {
        Self::clear_events(event);
        WifiEventFuture::new(event).await
    }

    /// Wait for multiple [`WifiEvent`]s.
    // 3385
    pub async fn wait_for_all_events(
        &mut self,
        mut events: EnumSet<WifiEvent>,
        clear_pending: bool,
    ) {
        if clear_pending {
            Self::clear_events(events);
        }

        while !events.is_empty() {
            let fired = MultiWifiEventFuture::new(events).await;
            events -= fired;
        }
    }

    // 3400
    fn apply_ap_config(&mut self, config: &AccessPointConfig) -> Result<(), WifiError> {
        self.ap_beacon_timeout = config.beacon_timeout;

        let mut cfg = wifi_config_t {
            ap: wifi_ap_config_t {
                ssid: [0; 32],
                password: [0; 64],
                ssid_len: 0,
                channel: config.channel,
                authmode: config.auth_method.to_raw(),
                ssid_hidden: if config.ssid_hidden { 1 } else { 0 },
                max_connection: config.max_connections as u8,
                beacon_interval: 100,
                pairwise_cipher: wifi_cipher_type_t_WIFI_CIPHER_TYPE_CCMP,
                ftm_responder: false,
                pmf_cfg: wifi_pmf_config_t {
                    capable: true,
                    required: false,
                },
                sae_pwe_h2e: 0,
                csa_count: 3,
                dtim_period: config.dtim_period,
                transition_disable: 0,
                sae_ext: 0,
                bss_max_idle_cfg: include::wifi_bss_max_idle_config_t {
                    period: 0,
                    protected_keep_alive: false,
                },
                gtk_rekey_interval: 0,
            },
        };

        if config.auth_method == AuthMethod::None && !config.password.is_empty() {
            return Err(WifiError::InternalError(
                InternalWifiError::EspErrInvalidArg,
            ));
        }

        unsafe {
            cfg.ap.ssid[0..(config.ssid.len())].copy_from_slice(config.ssid.as_bytes());
            cfg.ap.ssid_len = config.ssid.len() as u8;
            cfg.ap.password[0..(config.password.len())].copy_from_slice(config.password.as_bytes());
            esp_wifi_result!(esp_wifi_set_config(wifi_interface_t_WIFI_IF_AP, &mut cfg))
        }
    }

    // 3445
    fn apply_sta_config(&mut self, config: &ClientConfig) -> Result<(), WifiError> {
        let mut cfg = wifi_config_t {
            sta: wifi_sta_config_t {
                ssid: [0; 32],
                password: [0; 64],
                scan_method: config.scan_method as c_uint,
                bssid_set: config.bssid.is_some(),
                bssid: config.bssid.unwrap_or_default(),
                channel: config.channel.unwrap_or(0),
                listen_interval: config.listen_interval,
                sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
                threshold: wifi_scan_threshold_t {
                    rssi: -99,
                    authmode: config.auth_method.to_raw(),
                    rssi_5g_adjustment: 0,
                },
                pmf_cfg: wifi_pmf_config_t {
                    capable: true,
                    required: false,
                },
                sae_pwe_h2e: 3,
                _bitfield_align_1: [0; 0],
                _bitfield_1: __BindgenBitfieldUnit::new([0; 4]),
                failure_retry_cnt: config.failure_retry_cnt,
                _bitfield_align_2: [0; 0],
                _bitfield_2: __BindgenBitfieldUnit::new([0; 4]),
                sae_pk_mode: 0, // ??
                sae_h2e_identifier: [0; 32],
            },
        };

        if config.auth_method == AuthMethod::None && !config.password.is_empty() {
            return Err(WifiError::InternalError(
                InternalWifiError::EspErrInvalidArg,
            ));
        }

        unsafe {
            cfg.sta.ssid[0..(config.ssid.len())].copy_from_slice(config.ssid.as_bytes());
            cfg.sta.password[0..(config.password.len())]
                .copy_from_slice(config.password.as_bytes());

            esp_wifi_result!(esp_wifi_set_config(wifi_interface_t_WIFI_IF_STA, &mut cfg))
        }
    }
}

// 3630
impl WifiEvent {
    pub(crate) fn waker(&self) -> &'static AtomicWaker {
        // for now use only one waker for all events
        // if that ever becomes a problem we might want to pick some events to use their
        // own
        static WAKER: AtomicWaker = AtomicWaker::new();
        &WAKER
    }
}

// 3640
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub(crate) struct WifiEventFuture {
    event: WifiEvent,
}

// 3645
impl WifiEventFuture {
    /// Creates a new `Future` for the specified WiFi event.
    pub fn new(event: WifiEvent) -> Self {
        Self { event }
    }
}

// 3652
impl core::future::Future for WifiEventFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.event.waker().register(cx.waker());
        if WIFI_EVENTS.with(|events| events.remove(self.event)) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

// 3668
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub(crate) struct MultiWifiEventFuture {
    event: EnumSet<WifiEvent>,
}

// 3673
impl MultiWifiEventFuture {
    /// Creates a new `Future` for the specified set of WiFi events.
    pub fn new(event: EnumSet<WifiEvent>) -> Self {
        Self { event }
    }
}

// 3680
impl core::future::Future for MultiWifiEventFuture {
    type Output = EnumSet<WifiEvent>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let output = WIFI_EVENTS.with(|events| {
            let active = events.intersection(self.event);
            events.remove_all(active);
            active
        });
        if output.is_empty() {
            for event in self.event.iter() {
                event.waker().register(cx.waker());
            }

            Poll::Pending
        } else {
            Poll::Ready(output)
        }
    }
}
