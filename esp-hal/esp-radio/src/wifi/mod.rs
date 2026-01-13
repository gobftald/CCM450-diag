//! WiFi

// 3
pub mod event;
mod internal;
pub(crate) mod os_adapter;
pub(crate) mod state;

// 7
use alloc::{collections::vec_deque::VecDeque, string::String};
use core::{marker::PhantomData, ptr::addr_of, task::Poll};

// 17
use enumset::{EnumSet, EnumSetType};
use esp_hal::{asynch::AtomicWaker, sync::NonReentrantMutex};

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
    //EspWifiController, common_adapter::read_mac, esp_wifi_result, hal::ram,
    Controller,
    common_adapter::read_mac,
    esp_wifi_result,
    hal::ram,
    wifi::private::EspWifiPacketBuffer,
};

// 50
const MTU: usize = crate::CONFIG.mtu;

// 60
use crate::binary::{
    c_types,
    include::{
        self, __BindgenBitfieldUnit, esp_err_t, esp_interface_t_ESP_IF_WIFI_AP,
        esp_interface_t_ESP_IF_WIFI_STA, esp_supplicant_init, esp_wifi_connect,
        esp_wifi_disconnect, esp_wifi_get_mode, esp_wifi_init_internal,
        esp_wifi_internal_free_rx_buffer, esp_wifi_internal_reg_rxcb, esp_wifi_internal_tx,
        esp_wifi_set_config, esp_wifi_set_mode, esp_wifi_set_tx_done_cb, esp_wifi_start,
        g_wifi_default_wpa_crypto_funcs, wifi_ap_config_t, wifi_auth_mode_t,
        wifi_cipher_type_t_WIFI_CIPHER_TYPE_CCMP, wifi_config_t, wifi_interface_t,
        wifi_interface_t_WIFI_IF_AP, wifi_interface_t_WIFI_IF_STA, wifi_mode_t,
        wifi_mode_t_WIFI_MODE_AP, wifi_mode_t_WIFI_MODE_APSTA, wifi_mode_t_WIFI_MODE_NULL,
        wifi_mode_t_WIFI_MODE_STA, wifi_pmf_config_t, wifi_scan_threshold_t,
        wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL, wifi_sta_config_t,
    },
};

/// Supported Wi-Fi authentication methods.
#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
// 87
pub enum AuthMethod {
    /// No authentication (open network).
    None,

    /// Wired Equivalent Privacy (WEP) authentication.
    WEP,

    /// Wi-Fi Protected Access (WPA) authentication.
    WPA,

    /// Wi-Fi Protected Access 2 (WPA2) Personal authentication (default).
    #[default]
    WPA2Personal,

    /// WPA/WPA2 Personal authentication (supports both).
    WPAWPA2Personal,

    /// WPA2 Enterprise authentication.
    WPA2Enterprise,

    /// WPA3 Personal authentication.
    WPA3Personal,

    /// WPA2/WPA3 Personal authentication (supports both).
    WPA2WPA3Personal,

    /// WLAN Authentication and Privacy Infrastructure (WAPI).
    WAPIPersonal,
}

/// Supported Wi-Fi protocols.
#[derive(EnumSetType, Debug, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Default)]
// 122
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
#[derive(Clone, PartialEq, Eq)]
// 190
pub struct AccessPointConfiguration {
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
}

// 216
impl AccessPointConfiguration {
    fn validate(&self) -> Result<(), WifiError> {
        if self.ssid.len() > 32 {
            return Err(WifiError::InvalidArguments);
        }

        if self.password.len() > 64 {
            return Err(WifiError::InvalidArguments);
        }

        Ok(())
    }
}

// 230
impl Default for AccessPointConfiguration {
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
        }
    }
}

// 245
impl core::fmt::Debug for AccessPointConfiguration {
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
            .finish()
    }
}
#[cfg(feature = "defmt")]
// 261
impl defmt::Format for AccessPointConfiguration {
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
            }}",
            self.ssid.as_str(),
            self.ssid_hidden,
            self.channel,
            self.secondary_channel,
            protocol_set,
            self.auth_method,
            self.max_connections
        );
    }
}

/// Client configuration for a Wi-Fi connection.
#[derive(Clone, PartialEq, Eq, Default)]
// 307
pub struct ClientConfiguration {
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
}

// 325
impl ClientConfiguration {
    fn validate(&self) -> Result<(), WifiError> {
        if self.ssid.len() > 32 {
            return Err(WifiError::InvalidArguments);
        }

        if self.password.len() > 64 {
            return Err(WifiError::InvalidArguments);
        }

        Ok(())
    }
}

// 339
impl core::fmt::Debug for ClientConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClientConfiguration")
            .field("ssid", &self.ssid)
            .field("bssid", &self.bssid)
            .field("auth_method", &self.auth_method)
            .field("password", &"**REDACTED**")
            .field("channel", &self.channel)
            .finish()
    }
}

#[cfg(feature = "defmt")]
// 4352
impl defmt::Format for ClientConfiguration {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(
            fmt,
            "ClientConfiguration {{\
            ssid: {}, \
            bssid: {:?}, \
            auth_method: {:?}, \
            password: **REDACTED**, \
            channel: {:?}, \
            }}",
            self.ssid.as_str(),
            self.bssid,
            self.auth_method,
            self.channel
        )
    }
}

/// Introduces Wi-Fi configuration options.
#[derive(EnumSetType, Debug, PartialOrd)]
//#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 590
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
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 608
pub enum Configuration {
    /// No configuration (default).
    #[default]
    None,

    /// Client-only configuration.
    Client(ClientConfiguration),

    /// Access point-only configuration.
    AccessPoint(AccessPointConfiguration),

    /// Simultaneous client and access point configuration.
    Mixed(ClientConfiguration, AccessPointConfiguration),
    /*
    /// EAP client configuration for enterprise Wi-Fi.
    #[cfg_attr(feature = "serde", serde(skip))]
    EapClient(EapClientConfiguration),
    */
}

// 627
impl Configuration {
    // 628
    fn validate(&self) -> Result<(), WifiError> {
        match self {
            Configuration::None => Ok(()),
            Configuration::Client(client_configuration) => client_configuration.validate(),
            Configuration::AccessPoint(access_point_configuration) => {
                access_point_configuration.validate()
            }
            Configuration::Mixed(client_configuration, access_point_configuration) => {
                client_configuration.validate()?;
                access_point_configuration.validate()
            } /*
              Configuration::EapClient(eap_client_configuration) => {
                  eap_client_configuration.validate()
              }
              */
        }
    }
}

// 740
trait AuthMethodExt {
    fn to_raw(&self) -> wifi_auth_mode_t;
}

// 745
impl AuthMethodExt for AuthMethod {
    fn to_raw(&self) -> wifi_auth_mode_t {
        match self {
            AuthMethod::None => include::wifi_auth_mode_t_WIFI_AUTH_OPEN,
            AuthMethod::WEP => include::wifi_auth_mode_t_WIFI_AUTH_WEP,
            AuthMethod::WPA => include::wifi_auth_mode_t_WIFI_AUTH_WPA_PSK,
            AuthMethod::WPA2Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK,
            AuthMethod::WPAWPA2Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK,
            AuthMethod::WPA2Enterprise => include::wifi_auth_mode_t_WIFI_AUTH_WPA2_ENTERPRISE,
            AuthMethod::WPA3Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA3_PSK,
            AuthMethod::WPA2WPA3Personal => include::wifi_auth_mode_t_WIFI_AUTH_WPA2_WPA3_PSK,
            AuthMethod::WAPIPersonal => include::wifi_auth_mode_t_WIFI_AUTH_WAPI_PSK,
        }
    }
}

/// Wifi Mode (Sta and/or Ap)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 780
pub enum WifiMode {
    /// Station mode.
    Sta,
    /// Access Point mode.
    Ap,
    /// Both Station and Access Point modes.
    ApSta,
}

// 789
impl WifiMode {
    // 790
    pub(crate) fn current() -> Result<Self, WifiError> {
        let mut mode = wifi_mode_t_WIFI_MODE_NULL;
        esp_wifi_result!(unsafe { esp_wifi_get_mode(&mut mode) })?;

        Self::try_from(mode)
    }

    /// Returns true if this mode works as a client
    // 798
    pub fn is_sta(&self) -> bool {
        match self {
            Self::Sta | Self::ApSta => true,
            Self::Ap => false,
        }
    }

    /// Returns true if this mode works as an access point
    // 806
    pub fn is_ap(&self) -> bool {
        match self {
            Self::Sta => false,
            Self::Ap | Self::ApSta => true,
        }
    }
}

// 814
impl TryFrom<wifi_mode_t> for WifiMode {
    type Error = WifiError;

    /// Converts a `wifi_mode_t` C-type into a `WifiMode`.
    fn try_from(value: wifi_mode_t) -> Result<Self, Self::Error> {
        #[allow(non_upper_case_globals)]
        match value {
            wifi_mode_t_WIFI_MODE_STA => Ok(Self::Sta),
            wifi_mode_t_WIFI_MODE_AP => Ok(Self::Ap),
            wifi_mode_t_WIFI_MODE_APSTA => Ok(Self::ApSta),
            _ => Err(WifiError::UnknownWifiMode),
        }
    }
}

// 1044
const RX_QUEUE_SIZE: usize = crate::CONFIG.rx_queue_size;
const TX_QUEUE_SIZE: usize = crate::CONFIG.tx_queue_size;

// 1047
pub(crate) static DATA_QUEUE_RX_AP: NonReentrantMutex<VecDeque<EspWifiPacketBuffer>> =
    NonReentrantMutex::new(VecDeque::new());
pub(crate) static DATA_QUEUE_RX_STA: NonReentrantMutex<VecDeque<EspWifiPacketBuffer>> =
    NonReentrantMutex::new(VecDeque::new());

/// Common errors.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
// 1057
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
#[repr(i32)]
#[derive(Debug, FromPrimitive, EnumSetType)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1082
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
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1190
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

/// Get the STA MAC address
// 1317
pub fn sta_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 0);
    }
}

/// Get the AP MAC address
// 1259
pub fn ap_mac(mac: &mut [u8; 6]) {
    unsafe {
        read_mac(mac as *mut u8, 1);
    }
}

// 1272
pub(crate) fn wifi_init() -> Result<(), WifiError> {
    unsafe {
        internal::G_CONFIG.wpa_crypto_funcs = g_wifi_default_wpa_crypto_funcs;
        internal::G_CONFIG.feature_caps = internal::g_wifi_feature_caps;

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

        crate::flags::WIFI.store(true, Ordering::SeqCst);

        Ok(())
    }
}

// 1350
unsafe extern "C" fn recv_cb_sta(
    buffer: *mut c_types::c_void,
    len: u16,
    eb: *mut c_types::c_void,
) -> esp_err_t {
    let packet = EspWifiPacketBuffer { buffer, len, eb };
    // We must handle the result outside of the lock because
    // EspWifiPacketBuffer::drop must not be called in a critical section.
    // Dropping an EspWifiPacketBuffer will call `esp_wifi_internal_free_rx_buffer`
    // which will try to lock an internal mutex. If the mutex is already taken,
    // the function will try to trigger a context switch, which will fail if we
    // are in an interrupt-free context.
    match DATA_QUEUE_RX_STA.with(|queue| {
        if queue.len() < RX_QUEUE_SIZE {
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

// 1381
unsafe extern "C" fn recv_cb_ap(
    buffer: *mut c_types::c_void,
    len: u16,
    eb: *mut c_types::c_void,
) -> esp_err_t {
    let packet = EspWifiPacketBuffer { buffer, len, eb };
    // We must handle the result outside of the critical section because
    // EspWifiPacketBuffer::drop must not be called in a critical section.
    // Dropping an EspWifiPacketBuffer will call `esp_wifi_internal_free_rx_buffer`
    // which will try to lock an internal mutex. If the mutex is already taken,
    // the function will try to trigger a context switch, which will fail if we
    // are in an interrupt-free context.
    match DATA_QUEUE_RX_AP.with(|queue| {
        if queue.len() < RX_QUEUE_SIZE {
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

// 1412
pub(crate) static WIFI_TX_INFLIGHT: AtomicUsize = AtomicUsize::new(0);

// 1414
fn decrement_inflight_counter() {
    unwrap!(
        WIFI_TX_INFLIGHT.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
            Some(x.saturating_sub(1))
        })
    );
}

#[ram]
// 1423
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

// 1437
pub(crate) fn wifi_start() -> Result<(), WifiError> {
    unsafe {
        esp_wifi_result!(esp_wifi_start())?;

        let mode = WifiMode::current()?;

        // This is not an if-else because in AP-STA mode, both are true
        if mode.is_ap() {
            esp_wifi_result!(include::esp_wifi_set_inactive_time(
                wifi_interface_t_WIFI_IF_AP,
                crate::CONFIG.ap_beacon_timeout
            ))?;
        }
        if mode.is_sta() {
            esp_wifi_result!(include::esp_wifi_set_inactive_time(
                wifi_interface_t_WIFI_IF_STA,
                crate::CONFIG.beacon_timeout
            ))?;
        };
    }

    Ok(())
}

// 1605
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
    // 1617
    pub struct EspWifiPacketBuffer {
        pub(crate) buffer: *mut c_types::c_void,
        pub(crate) len: u16,
        pub(crate) eb: *mut c_types::c_void,
    }

    // 1623
    unsafe impl Send for EspWifiPacketBuffer {}

    // 1625
    impl Drop for EspWifiPacketBuffer {
        fn drop(&mut self) {
            trace!("Dropping EspWifiPacketBuffer, freeing memory");
            unsafe { esp_wifi_internal_free_rx_buffer(self.eb) };
        }
    }

    // 1632
    impl EspWifiPacketBuffer {
        pub fn as_slice_mut(&mut self) -> &mut [u8] {
            unsafe { core::slice::from_raw_parts_mut(self.buffer as *mut u8, self.len as usize) }
        }
    }
}

/// Provides methods for retrieving the Wi-Fi mode and MAC address.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// 1641
pub enum WifiDeviceMode {
    Sta,
    Ap,
}

// 1646
impl WifiDeviceMode {
    // 1647
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

    // 1662
    fn data_queue_rx(&self) -> &'static NonReentrantMutex<VecDeque<EspWifiPacketBuffer>> {
        match self {
            WifiDeviceMode::Sta => &DATA_QUEUE_RX_STA,
            WifiDeviceMode::Ap => &DATA_QUEUE_RX_AP,
        }
    }

    // 1669
    fn can_send(&self) -> bool {
        WIFI_TX_INFLIGHT.load(Ordering::SeqCst) < TX_QUEUE_SIZE
    }

    // 1673
    fn increase_in_flight_counter(&self) {
        WIFI_TX_INFLIGHT.fetch_add(1, Ordering::SeqCst);
    }

    // 1677
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

    // 1689
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

    // 1704
    fn interface(&self) -> wifi_interface_t {
        match self {
            WifiDeviceMode::Sta => wifi_interface_t_WIFI_IF_STA,
            WifiDeviceMode::Ap => wifi_interface_t_WIFI_IF_AP,
        }
    }

    // 1768
    fn register_transmit_waker(&self, cx: &mut core::task::Context<'_>) {
        embassy::TRANSMIT_WAKER.register(cx.waker())
    }

    // 1711
    fn register_receive_waker(&self, cx: &mut core::task::Context<'_>) {
        match self {
            WifiDeviceMode::Sta => embassy::STA_RECEIVE_WAKER.register(cx.waker()),
            WifiDeviceMode::Ap => embassy::AP_RECEIVE_WAKER.register(cx.waker()),
        }
    }

    // 1722
    fn register_link_state_waker(&self, cx: &mut core::task::Context<'_>) {
        match self {
            WifiDeviceMode::Sta => embassy::STA_LINK_STATE_WAKER.register(cx.waker()),
            WifiDeviceMode::Ap => embassy::AP_LINK_STATE_WAKER.register(cx.waker()),
        }
    }

    // 1729
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
// 1750
pub struct WifiDevice<'d> {
    _phantom: PhantomData<&'d ()>,
    mode: WifiDeviceMode,
}

// 1755
impl WifiDevice<'_> {
    /// Retrieves the MAC address of the Wi-Fi device.
    // 1757
    pub fn mac_address(&self) -> [u8; 6] {
        self.mode.mac_address()
    }
}

#[derive(Debug)]
// 2097
pub struct WifiRxToken {
    mode: WifiDeviceMode,
}

// 2101
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

#[derive(Debug)]
// 2140
pub struct WifiTxToken {
    mode: WifiDeviceMode,
}

// 2144
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
// 2182
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

// 2199
fn apply_ap_config(config: &AccessPointConfiguration) -> Result<(), WifiError> {
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
            dtim_period: 2,
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

// 2237
fn apply_sta_config(config: &ClientConfiguration) -> Result<(), WifiError> {
    let mut cfg = wifi_config_t {
        sta: wifi_sta_config_t {
            ssid: [0; 32],
            password: [0; 64],
            scan_method: crate::CONFIG.scan_method,
            bssid_set: config.bssid.is_some(),
            bssid: config.bssid.unwrap_or_default(),
            channel: config.channel.unwrap_or(0),
            listen_interval: crate::CONFIG.listen_interval,
            sort_method: wifi_sort_method_t_WIFI_CONNECT_AP_BY_SIGNAL,
            threshold: wifi_scan_threshold_t {
                rssi: -99,
                authmode: config.auth_method.to_raw(),
            },
            pmf_cfg: wifi_pmf_config_t {
                capable: true,
                required: false,
            },
            sae_pwe_h2e: 3,
            _bitfield_align_1: [0; 0],
            _bitfield_1: __BindgenBitfieldUnit::new([0; 4]),
            failure_retry_cnt: crate::CONFIG.failure_retry_cnt,
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
        cfg.sta.password[0..(config.password.len())].copy_from_slice(config.password.as_bytes());

        esp_wifi_result!(esp_wifi_set_config(wifi_interface_t_WIFI_IF_STA, &mut cfg))
    }
}

// 2415
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

#[macro_export]
// 2424
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

// 2439
pub(crate) mod embassy {
    use embassy_net_driver::{Capabilities, Driver, HardwareAddress, RxToken, TxToken};

    use super::*;

    // We can get away with a single tx waker because the transmit queue is shared
    // between interfaces.
    // 2447
    pub(crate) static TRANSMIT_WAKER: AtomicWaker = AtomicWaker::new();

    // 2449
    pub(crate) static AP_RECEIVE_WAKER: AtomicWaker = AtomicWaker::new();
    pub(crate) static AP_LINK_STATE_WAKER: AtomicWaker = AtomicWaker::new();

    // 2452
    pub(crate) static STA_RECEIVE_WAKER: AtomicWaker = AtomicWaker::new();
    pub(crate) static STA_LINK_STATE_WAKER: AtomicWaker = AtomicWaker::new();

    // 2455
    impl RxToken for WifiRxToken {
        fn consume<R, F>(self, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            self.consume_token(f)
        }
    }

    // 2464
    impl TxToken for WifiTxToken {
        fn consume<R, F>(self, len: usize, f: F) -> R
        where
            F: FnOnce(&mut [u8]) -> R,
        {
            self.consume_token(len, f)
        }
    }

    // 2473
    impl Driver for WifiDevice<'_> {
        type RxToken<'a>
            = WifiRxToken
        where
            Self: 'a;
        type TxToken<'a>
            = WifiTxToken
        where
            Self: 'a;

        // 2483
        fn receive(
            &mut self,
            cx: &mut core::task::Context<'_>,
        ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
            self.mode.register_receive_waker(cx);
            self.mode.register_transmit_waker(cx);
            self.mode.rx_token()
        }

        // 2492
        fn transmit(&mut self, cx: &mut core::task::Context<'_>) -> Option<Self::TxToken<'_>> {
            self.mode.register_transmit_waker(cx);
            self.mode.tx_token()
        }

        // 2497
        fn link_state(
            &mut self,
            cx: &mut core::task::Context<'_>,
        ) -> embassy_net_driver::LinkState {
            self.mode.register_link_state_waker(cx);
            self.mode.link_state()
        }

        // 2505
        fn capabilities(&self) -> Capabilities {
            let mut caps = Capabilities::default();
            caps.max_transmission_unit = MTU;
            caps.max_burst_size = if crate::CONFIG.max_burst_size == 0 {
                None
            } else {
                Some(crate::CONFIG.max_burst_size)
            };
            caps
        }

        // 2516
        fn hardware_address(&self) -> HardwareAddress {
            HardwareAddress::Ethernet(self.mac_address())
        }
    }
}

#[non_exhaustive]
// 2543
pub struct Interfaces<'d> {
    pub sta: WifiDevice<'d>,
    pub ap: WifiDevice<'d>,
    //#[cfg(feature = "esp-now")]
    //pub esp_now: crate::esp_now::EspNow<'d>,
    //#[cfg(feature = "sniffer")]
    //pub sniffer: Sniffer,
}

/// Create a WiFi controller and it's associated interfaces.
///
/// Dropping the controller will deinitialize / stop WiFi.
///
/// Make sure to **not** call this function while interrupts are disabled.
// 2557
pub fn new<'d>(
    //_inited: &'d EspWifiController<'d>,
    _inited: &'d Controller<'d>,
    _device: crate::hal::peripherals::WIFI<'d>,
) -> Result<(WifiController<'d>, Interfaces<'d>), WifiError> {
    if crate::is_interrupts_disabled() {
        return Err(WifiError::Unsupported);
    }

    let controller = WifiController {
        _phantom: Default::default(),
    };

    crate::wifi::wifi_init()?;

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

#[non_exhaustive]
// 2614
pub struct WifiController<'d> {
    _phantom: PhantomData<&'d ()>,
}

// 2626
impl WifiController<'_> {
    /// Set the configuration.
    ///
    /// This will set the mode accordingly.
    /// You need to use Wifi::connect() for connecting to an AP.
    ///
    /// Passing [Configuration::None] will disable both, AP and STA mode.
    ///
    /// If you don't intent to use WiFi anymore at all consider tearing down
    /// WiFi completely.
    // 2781
    pub fn set_configuration(&mut self, conf: &Configuration) -> Result<(), WifiError> {
        conf.validate()?;

        let mode = match conf {
            Configuration::None => wifi_mode_t_WIFI_MODE_NULL,
            Configuration::Client(_) => wifi_mode_t_WIFI_MODE_STA,
            Configuration::AccessPoint(_) => wifi_mode_t_WIFI_MODE_AP,
            Configuration::Mixed(_, _) => wifi_mode_t_WIFI_MODE_APSTA,
            //Configuration::EapClient(_) => wifi_mode_t_WIFI_MODE_STA,
        };

        esp_wifi_result!(unsafe { esp_wifi_set_mode(mode) })?;

        match conf {
            Configuration::None => Ok::<(), WifiError>(()),
            Configuration::Client(config) => apply_sta_config(config),
            Configuration::AccessPoint(config) => apply_ap_config(config),
            Configuration::Mixed(sta_config, ap_config) => {
                apply_ap_config(ap_config).and_then(|()| apply_sta_config(sta_config))
            } //Configuration::EapClient(config) => apply_sta_eap_config(config),
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

    /// Get the supported capabilities of the controller.
    // 2766
    pub fn capabilities(&self) -> Result<EnumSet<crate::wifi::Capability>, WifiError> {
        //pub fn capabilities(&self) -> Result<EnumSet<crate::wifi::Capability>, InternalWifiError> {
        let caps =
            enumset::enum_set! { Capability::Client | Capability::AccessPoint | Capability::Mixed };
        //InternalWifiError::EspErrNoMem;
        Ok(caps)
        //Err(caps)
    }

    // 2825
    fn connect_impl(&mut self) -> Result<(), WifiError> {
        esp_wifi_result!(unsafe { esp_wifi_connect() })
    }

    // 2829
    pub fn disconnect_impl(&mut self) -> Result<(), WifiError> {
        esp_wifi_result!(unsafe { esp_wifi_disconnect() })
    }

    // 2863
    fn mode(&self) -> Result<WifiMode, WifiError> {
        WifiMode::current()
    }

    /// Async version of [`crate::wifi::WifiController`]'s `start` method
    // 2904
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

        wifi_start()?;

        self.wait_for_all_events(events, false).await;

        Ok(())
    }

    /// Async version of [`crate::wifi::WifiController`]'s `connect` method
    // 2949
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

    // 2981
    fn clear_events(events: impl Into<EnumSet<WifiEvent>>) {
        WIFI_EVENTS.with(|evts| evts.get_mut().remove_all(events.into()));
    }

    /// Wait for one [`WifiEvent`].
    // 2986
    pub async fn wait_for_event(&mut self, event: WifiEvent) {
        Self::clear_events(event);
        WifiEventFuture::new(event).await
    }

    /// Wait for multiple [`WifiEvent`]s.
    // 3005
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
}

// 3021
impl WifiEvent {
    pub(crate) fn waker(&self) -> &'static AtomicWaker {
        // for now use only one waker for all events
        // if that ever becomes a problem we might want to pick some events to use their
        // own
        static WAKER: AtomicWaker = AtomicWaker::new();
        &WAKER
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
// 3032
pub(crate) struct WifiEventFuture {
    event: WifiEvent,
}

// 3036
impl WifiEventFuture {
    /// Creates a new `Future` for the specified WiFi event.
    pub fn new(event: WifiEvent) -> Self {
        Self { event }
    }
}

// 3043
impl core::future::Future for WifiEventFuture {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.event.waker().register(cx.waker());
        if WIFI_EVENTS.with(|events| events.get_mut().remove(self.event)) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
// 3060
pub(crate) struct MultiWifiEventFuture {
    event: EnumSet<WifiEvent>,
}

// 3064
impl MultiWifiEventFuture {
    /// Creates a new `Future` for the specified set of WiFi events.
    pub fn new(event: EnumSet<WifiEvent>) -> Self {
        Self { event }
    }
}

// 3071
impl core::future::Future for MultiWifiEventFuture {
    type Output = EnumSet<WifiEvent>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let output = WIFI_EVENTS.with(|events| {
            let events = events.get_mut();
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
