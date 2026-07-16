use core::net::Ipv4Addr;

use embassy_net::tcp::TcpSocket;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_futures::select::{select, Either};
use embedded_io_async::Write;

use crate::gps::{GPS_UPDATED, GGA_MSG, GGA_SIZE, RMC_MSG, RMC_SIZE};

pub static mut TCP_STAT: [u8; 1] = [b'D'];
pub static mut TCP_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

struct WifiCandidate {
    ssid: &'static str,
    password: &'static str,
    priority: u8,
    discover_timeout: u64,      // only relevant for DHCP
    static_ip: Option<(Ipv4Addr, u8, Ipv4Addr)>, // (address, prefix_len, gateway)
}

const CANDIDATES: [WifiCandidate; 2] = [
    WifiCandidate {
        ssid: env!("TETH_SSID"),
        password: env!("TETH_PWD"),
        priority: 2,                    // higher priority
        discover_timeout: 1,
        static_ip: None,
    },
    WifiCandidate {
        ssid: env!("HOME_SSID"),
        password: env!("HOME_PWD"),
        priority: 1,                    // lower priority
        discover_timeout: 10,
        static_ip: Some((
            parse_ipv4(env!("HOME_FIX_IP")),
            24,
            parse_ipv4(env!("HOME_ROUTER")),
        )),
    },
];

#[embassy_executor::task]
pub async fn connection_task(
    mut controller: esp_radio::wifi::WifiController<'static>,
    ap_config: esp_radio::wifi::AccessPointConfig,
    sta_stack: embassy_net::Stack<'static>,
    wifi_rescan_request: &'static Signal<NoopRawMutex, ()>,
    sta_state_changed: &'static Signal<NoopRawMutex, ()>,
) {
    use esp_radio::wifi::{WifiEvent, ScanConfig, ClientConfig, ModeConfig};

    unwrap!(controller.start_async().await);
    trace!("*** WiFi: AP+STA started");

    loop {
        trace!("*** WiFi STA: Scanning...");
        let target = match controller.scan_with_config_async(
            ScanConfig::default()
        ).await
        {
            Ok(aps) => {
                CANDIDATES.iter()
                    .filter(|wc|
                        aps.iter()
                            .any(|ap| ap.ssid.as_str() == wc.ssid)
                    )
                    .max_by_key(|wc| wc.priority)
            }
            Err(e) => {
                warn!("*** WiFi STA: Scan failed: {:?}", &e);
                None
            }
        };

        let Some(target) = target else {
            trace!("*** WiFi STA: No known SSID visible, retrying scan in 5s...");
            embassy_time::Timer::after_secs(5).await;
            continue;
        };

        trace!("*** WiFi STA: Selected {} (priority {})", target.ssid, target.priority);

        let client_config = ClientConfig::default()
            .with_ssid((target.ssid).into())
            .with_password((target.password).into());

        if let Err(e) = controller.set_config(
            &ModeConfig::ApSta(client_config, ap_config.clone())) {
            warn!("*** WiFi: set_config failed: {:?}", &e);
            embassy_time::Timer::after_secs(5).await;
            continue;
        }

        match target.static_ip {
            Some((addr, prefix_len, gateway)) => {
                // Clear the stack and force Static config
                sta_stack.set_config_v4(embassy_net::ConfigV4::None);
                sta_stack.set_config_v4(
                    embassy_net::ConfigV4::Static(
                        embassy_net::StaticConfigV4 {
                            address: embassy_net::Ipv4Cidr::new(addr, prefix_len),
                            gateway: Some(gateway),
                            dns_servers: Default::default(),
                        }
                    )
                );
            }
            None => {
                // Clear the stack and force DHCP mode BEFORE the hardware connects
                sta_stack.set_config_v4(embassy_net::ConfigV4::None);
                let mut dhcp_config = embassy_net::DhcpConfig::default();
                dhcp_config.retry_config.discover_timeout =
                    core::time::Duration::from_secs(target.discover_timeout).into();
                sta_stack.set_config_v4(embassy_net::ConfigV4::Dhcp(dhcp_config));
            }
        }

        trace!("*** WiFi STA: Connecting...");
        match controller.connect_async().await {
            Ok(()) => {
                trace!("*** WiFi STA: Connected!");
                unsafe { TCP_STAT[0] = b'U'; }

                // Wait for Layer-3 configuration to settle.
                match target.static_ip {
                    Some(_) => {
                        // With static config, poll briefly until the stack updates
                        loop {
                            if let Some(cfg) = sta_stack.config_v4() {
                                info!("*** WiFi STA: Static IP: {}", cfg.address);
                                unsafe { TCP_ADDR = cfg.address.address(); }
                                break;
                            }
                            embassy_time::Timer::after_millis(50).await;
                        }
                    }
                    None => {
                        // DHCP Mode: Wait cleanly for the events.
                        sta_stack.wait_link_up().await; 
                        sta_stack.wait_config_up().await;

                        if let Some(cfg) = sta_stack.config_v4() {
                            info!("*** WiFi STA: DHCP IP: {}", cfg.address);
                            unsafe { TCP_ADDR = cfg.address.address(); }
                        }
                    }
                }

                let was_forced = match select(
                    controller.wait_for_event(WifiEvent::StaDisconnected),
                    wifi_rescan_request.wait(),
                ).await {
                    Either::First(()) => {
                        trace!("*** WiFi STA: Disconnected, rescanning...");
                        sta_state_changed.signal(());
                        false
                    }
                    Either::Second(()) => {
                        trace!("*** WiFi: Manual rescan requested, power-cycling radio...");
                        true
                    }
                };

                // Force clear the embassy-net interface state
                let empty_static = embassy_net::ConfigV4::Static(embassy_net::StaticConfigV4 {
                    address: embassy_net::Ipv4Cidr::new(
                        embassy_net::Ipv4Address::from_octets([0, 0, 0, 0]),
                        0
                    ),
                    gateway: None,
                    dns_servers: Default::default(),
                });
                debug!("after let empty_static");
                sta_stack.set_config_v4(empty_static);
                debug!("after sta_stack.set_config_v4(empty_static);");

                // Complete hardware shutdown of the co-processor radio (AP + STA)
                let _ = controller.stop_async().await;
                debug!("after let _ = controller.stop_async().await;");

                // Clear transient hardware energy
                embassy_time::Timer::after_millis(300).await;
                unsafe { TCP_STAT[0] = b'D'; }

                // Cold-reboot the radio module
                trace!("*** WiFi: Hardware radio rebooting...");
                unwrap!(controller.start_async().await);
                trace!("*** WiFi: Radio recovered and freshly re-initialized!");

                // Signal TFTP server to drop its current sockets
                sta_state_changed.signal(());

                // When range or other issue, don't aggressively spin the CPU
                // in a continuous high-power scanning loop 
                if was_forced == false {
                    embassy_time::Timer::after_secs(5).await;
                }
            }
            Err(e) => {
                warn!("*** WiFi STA: Connect failed: {:?}", &e);
                embassy_time::Timer::after_millis(500).await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn tcp_task(
    sta_stack: embassy_net::Stack<'static>,
    sta_state_changed: &'static Signal<NoopRawMutex, ()>,
) {
    let mut rx_buf = [0u8; 4];  // we only send
    let mut tx_buf = [0u8; 512];
    let mut gps_updated = unwrap!(GPS_UPDATED.receiver());

    loop {
        loop {
            if sta_stack.is_link_up() { break; }
            embassy_time::Timer::after_millis(200).await;
            trace!("link is not up");
        }

        // make sure that no peding signal
        let _ = sta_state_changed.try_take();

        let mut socket =
            TcpSocket::new(sta_stack, &mut rx_buf, &mut tx_buf);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(5)));
        socket.set_keep_alive(Some(embassy_time::Duration::from_secs(10)));

        trace!("*** TCP NMEA: Waiting for GNSS Master connection...");

        let port = match env!("NMEA_PORT").parse::<u16>() {
            Ok(port) => port,
            Err(_) => 0
        };

        match select(
            socket.accept(embassy_net::IpListenEndpoint {addr: None, port }),
            sta_state_changed.wait(),
        ).await {
            Either::First(Ok(())) => {
                trace!("*** TCP NMEA: Client connected from {:?}", socket.remote_endpoint());
                unsafe { TCP_STAT[0] = b'C'; }
            }
            Either::First(Err(e)) => {
                warn!("*** TCP NMEA: Accept error: {:?}", e);

                embassy_time::Timer::after_secs(1).await;
                continue;
            }
            Either::Second(()) => {
                trace!("*** TCP NMEA: STA state changed while waiting, restarting listen");
                socket.close();
                continue;
            }
        }

        loop {
            gps_updated.changed().await;

            unsafe {
                match socket.write_all(&GGA_MSG[..GGA_SIZE]).await {
                    Ok(()) => trace!("*** TCP NMEA: Sent: {:a}", &GGA_MSG[..GGA_SIZE]),
                    Err(e) => {
                        warn!("*** TCP NMEA: Write error: {:?}", e);
                        break;
                    }
                }

                match socket.write_all(&RMC_MSG[..RMC_SIZE]).await {
                    Ok(()) => trace!("*** TCP NMEA: Sent: {:a}", &RMC_MSG[..RMC_SIZE]),
                    Err(e) => {
                        warn!("*** TCP NMEA: Write error: {:?}", e);
                        break;
                    }
                }
            }

            match select(socket.flush(), sta_state_changed.wait()).await {
                Either::First(Ok(())) => trace!("*** TCP NMEA: ACKed"),
                Either::First(Err(e)) => {
                    warn!("*** TCP NMEA: Flush/ACK error: {:?}", e);
                    break;
                }
                Either::Second(()) => {
                    trace!("*** TCP NMEA: STA state changed, closing socket");
                    break;
                }
            }
        }

        socket.close();

        embassy_time::Timer::after_millis(100).await;
        trace!("*** TCP NMEA: Connection closed, back to listening.");
    }
}

const fn parse_ipv4(s: &str) -> Ipv4Addr {
    let bytes = s.as_bytes();
    let mut octets = [0u8; 4];
    let mut octet_idx = 0;
    let mut current: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            octets[octet_idx] = current as u8;
            octet_idx += 1;
            current = 0;
        } else {
            current = current * 10 + (b - b'0') as u32;
        }
        i += 1;
    }
    octets[octet_idx] = current as u8;
    Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3])
}
