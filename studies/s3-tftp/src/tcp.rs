use core::net::Ipv4Addr;

use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;

use crate::gps::{GPS_UPDATED, GGA_MSG, GGA_SIZE, RMC_MSG, RMC_SIZE};

const NMEA_PORT: Option<&'static str> = option_env!("NMEA_PORT");

pub static mut TCP_STAT: [u8; 1] = [b'D'];
pub static mut TCP_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

#[embassy_executor::task]
pub async fn tcp_task(sta_stack: embassy_net::Stack<'static>) {

    // Wait for DHCP
    loop {
        if sta_stack.is_link_up() { break; }
        embassy_time::Timer::after_millis(1000).await;
        debug!("link is not up");
    }
    sta_stack.wait_config_up().await;
    if let Some(cfg) = sta_stack.config_v4() {
        info!("*** TCP NMEA: IP: {}", cfg.address);
        unsafe { TCP_ADDR = cfg.address.address(); }
    }

    let mut rx_buf = [0u8; 4];  // we only send
    let mut tx_buf = [0u8; 512];
    let mut gps_updated = unwrap!(GPS_UPDATED.receiver());

    loop {
        let mut socket =
            TcpSocket::new(sta_stack, &mut rx_buf, &mut tx_buf);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(30)));
        socket.set_keep_alive(Some(embassy_time::Duration::from_secs(10)));

        trace!("*** TCP NMEA: Waiting for GNSS Master connection...");

        let port = match env!("NMEA_PORT").parse::<u16>() {
            Ok(port) => port,
            Err(_) => 0
        };

        match socket.accept(
            embassy_net::IpListenEndpoint {
                addr: None,
                port: port,
            }
        ).await {
            Ok(()) => {
                info!("*** TCP NMEA: Client connected from {:?}", socket.remote_endpoint());
                unsafe { TCP_STAT[0] = b'C'; }
            }
            Err(e) => {
                warn!("*** TCP NMEA: Accept error: {:?}", e);
                unsafe { TCP_STAT[0] = b'D'; }

                embassy_time::Timer::after_secs(1).await;
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

            match socket.flush().await {
                Ok(()) => trace!("*** TCP NMEA: ACKed"),
                Err(e) => { warn!("*** TCP NMEA: Flush/ACK error: {:?}", e); break; }
            }
        }

        socket.close();
        unsafe { TCP_STAT[0] = b'D'; }

        embassy_time::Timer::after_millis(100).await;
        trace!("*** TCP NMEA: Connection closed, back to listening.");
    }
}
