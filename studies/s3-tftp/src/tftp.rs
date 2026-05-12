use embassy_net::udp::{UdpSocket, PacketMetadata};
use embassy_net::Stack;
use crate::sd_card::{SD_VOLUME_MGR, SharedVolumeManager};
use embedded_sdmmc::{Mode, VolumeIdx, Error as SdError};
use embassy_time::{Duration, Timer};

pub(crate) mod protocol {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum OpCode {
        Rrq = 1,
        Wrq = 2,
        Data = 3,
        Ack = 4,
        Error = 5,
    }

    impl From<u16> for OpCode {
        fn from(val: u16) -> Self {
            match val {
                1 => OpCode::Rrq,
                2 => OpCode::Wrq,
                3 => OpCode::Data,
                4 => OpCode::Ack,
                5 => OpCode::Error,
                _ => OpCode::Error, 
            }
        }
    }
}

use protocol::OpCode;

#[embassy_executor::task]
pub async fn tftp_task(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut [PacketMetadata::EMPTY; 2],
        &mut tx_buffer,
    );

    socket.bind(69).unwrap();

    let volume_mgr = SD_VOLUME_MGR.wait().await;

    loop {
        let mut buf = [0u8; 516];
        match socket.recv_from(&mut buf).await {
            Ok((n, remote_endpoint)) => {
                if n < 2 { continue; }
                let opcode = OpCode::from(u16::from_be_bytes([buf[0], buf[1]]));
                match opcode {
                    OpCode::Rrq => {
                        handle_rrq(&mut socket, remote_endpoint, &buf[2..n], volume_mgr).await;
                    }
                    OpCode::Wrq => {
                        handle_wrq(&mut socket, remote_endpoint, &buf[2..n], volume_mgr).await;
                    }
                    _ => {
                        send_error(&mut socket, remote_endpoint, 4, "Illegal TFTP operation").await;
                    }
                }
            }
            Err(e) => {
                error!("UDP recv error: {:?}", e);
            }
        }
    }
}

async fn handle_rrq(
    socket: &mut UdpSocket<'_>,
    remote: embassy_net::IpEndpoint,
    data: &[u8],
    volume_mgr: &'static SharedVolumeManager,
) {
    let (filename, _mode) = parse_request(data);
    info!("RRQ: {}", filename);

    let mut volume_mgr = volume_mgr.lock().await;
    let mut volume = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            send_error(socket, remote, 1, "Volume error").await;
            return;
        }
    };

    let mut root_dir = match volume.open_root_dir() {
        Ok(d) => d,
        Err(e) => {
            send_error(socket, remote, 1, "Root dir error").await;
            return;
        }
    };

    let mut file = match root_dir.open_file_in_dir(filename, Mode::ReadOnly) {
        Ok(f) => f,
        Err(e) => {
            send_error(socket, remote, 1, "File not found").await;
            return;
        }
    };

    let mut block_num = 1u16;
    let mut data_buf = [0u8; 512];
    
    loop {
        let n = match file.read(&mut data_buf) {
            Ok(n) => n,
            Err(e) => {
                send_error(socket, remote, 2, "Read error").await;
                return;
            }
        };

        let mut packet = [0u8; 516];
        packet[0..2].copy_from_slice(&(OpCode::Data as u16).to_be_bytes());
        packet[2..4].copy_from_slice(&block_num.to_be_bytes());
        packet[4..4+n].copy_from_slice(&data_buf[..n]);

        // Send and wait for ACK
        let mut retry = 0;
        loop {
            if let Err(e) = socket.send_to(&packet[..4+n], remote).await {
                error!("UDP send error: {:?}", e);
                return;
            }

            let mut ack_buf = [0u8; 4];
            match embassy_futures::select::select(
                socket.recv_from(&mut ack_buf),
                Timer::after(Duration::from_secs(2))
            ).await {
                embassy_futures::select::Either::First(Ok((an, _))) => {
                    if an >= 4 {
                        let op = OpCode::from(u16::from_be_bytes([ack_buf[0], ack_buf[1]]));
                        let b = u16::from_be_bytes([ack_buf[2], ack_buf[3]]);
                        if op == OpCode::Ack && b == block_num {
                            break; // Got correct ACK
                        }
                    }
                }
                _ => {
                    retry += 1;
                    if retry > 3 {
                        error!("TFTP timeout");
                        return;
                    }
                    info!("TFTP retry {}", retry);
                }
            }
        }

        block_num = block_num.wrapping_add(1);
        if n < 512 {
            break; // EOF
        }
    }
    info!("RRQ finished: {}", filename);
}

async fn handle_wrq(
    socket: &mut UdpSocket<'_>,
    remote: embassy_net::IpEndpoint,
    data: &[u8],
    volume_mgr: &'static SharedVolumeManager,
) {
    let (filename, _mode) = parse_request(data);
    info!("WRQ: {}", filename);

    let mut volume_mgr = volume_mgr.lock().await;
    let mut volume = match volume_mgr.open_volume(VolumeIdx(0)) {
        Ok(v) => v,
        Err(e) => {
            send_error(socket, remote, 1, "Volume error").await;
            return;
        }
    };

    let mut root_dir = match volume.open_root_dir() {
        Ok(d) => d,
        Err(e) => {
            send_error(socket, remote, 1, "Root dir error").await;
            return;
        }
    };

    let mut file = match root_dir.open_file_in_dir(filename, Mode::ReadWriteCreateOrOverwrite) {
        Ok(f) => f,
        Err(e) => {
            send_error(socket, remote, 1, "File create error").await;
            return;
        }
    };

    let mut block_num = 0u16;
    
    // Send initial ACK for WRQ
    let ack = (OpCode::Ack as u16).to_be_bytes();
    let b0 = block_num.to_be_bytes();
    let mut initial_ack = [0u8; 4];
    initial_ack[0..2].copy_from_slice(&ack);
    initial_ack[2..4].copy_from_slice(&b0);
    socket.send_to(&initial_ack, remote).await.ok();

    loop {
        let mut data_buf = [0u8; 516];
        match embassy_futures::select::select(
            socket.recv_from(&mut data_buf),
            Timer::after(Duration::from_secs(5))
        ).await {
            embassy_futures::select::Either::First(Ok((n, _))) => {
                if n < 4 { continue; }
                let op = OpCode::from(u16::from_be_bytes([data_buf[0], data_buf[1]]));
                let b = u16::from_be_bytes([data_buf[2], data_buf[3]]);
                
                if op == OpCode::Data && b == block_num.wrapping_add(1) {
                    block_num = b;
                    let payload = &data_buf[4..n];
                    if let Err(_) = file.write(payload) {
                        send_error(socket, remote, 3, "Write failed").await;
                        return;
                    }
                    
                    let mut ack_pkt = [0u8; 4];
                    ack_pkt[0..2].copy_from_slice(&(OpCode::Ack as u16).to_be_bytes());
                    ack_pkt[2..4].copy_from_slice(&block_num.to_be_bytes());
                    socket.send_to(&ack_pkt, remote).await.ok();

                    if payload.len() < 512 {
                        break; // EOF
                    }
                }
            }
            _ => {
                error!("WRQ timeout");
                return;
            }
        }
    }
    info!("WRQ finished: {}", filename);
}

fn parse_request(data: &[u8]) -> (&str, &str) {
    let mut parts = data.split(|&b| b == 0);
    let filename = core::str::from_utf8(parts.next().unwrap_or(&[])).unwrap_or("");
    let mode = core::str::from_utf8(parts.next().unwrap_or(&[])).unwrap_or("");
    (filename, mode)
}

async fn send_error(socket: &mut UdpSocket<'_>, remote: embassy_net::IpEndpoint, code: u16, msg: &str) {
    let mut buf = [0u8; 512];
    buf[0..2].copy_from_slice(&(OpCode::Error as u16).to_be_bytes());
    buf[2..4].copy_from_slice(&code.to_be_bytes());
    let msg_bytes = msg.as_bytes();
    let len = msg_bytes.len().min(512 - 5);
    buf[4..4+len].copy_from_slice(&msg_bytes[..len]);
    buf[4+len] = 0;
    socket.send_to(&buf[..5+len], remote).await.ok();
}
