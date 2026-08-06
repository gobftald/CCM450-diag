//! Minimal read-only TFTP server (RFC 1350 subset) built directly on
//! `embassy-net`, with no dependency on `smoltcp` types or the `smolapps`
//! crate. This avoids the version-drift / type-mismatch problems that come
//! from mixing a raw-smoltcp-oriented crate with embassy-net's own wrapper
//! types.
//!
//! Scope (deliberately minimal, matching a log-retrieval use case):
//! - RRQ (read) only. WRQ is rejected with an ERROR packet.
//! - `octet` (binary) transfer mode only — no netascii translation.
//! - Standard 512-byte blocks, no RFC 2347 option negotiation (blksize etc).
//! - One transfer served to completion at a time, on a single socket bound
//!   to port 69. This departs from strict RFC 1350 (which has the server
//!   reply from a fresh ephemeral port per transfer) but is far simpler and
//!   is fine for "pull the logs off the device" flow.
//! - A virtual index file requested as a bare `YYMMDD` (six digits, no
//!   extension, e.g. `260701`): lists every calendar day with data whose
//!   date is strictly after the requested one, one `YYMMDD` per line,
//!   nothing else — via `FileSource::list`. Use an early date (e.g.
//!   `000101`) to list everything. Fetch a specific day's data with
//!   `YYMMDD.raw` (real file, via `open`/`read_at`) or `YYMMDD.gpx`
//!   (generated on the fly, via `FileSource::start_gpx`/`next_gpx_chunk`
//!   — see the companion `gpx` module).
//! - Filenames are matched case-sensitively (lowercase only)
//!

use embassy_net::{Stack, udp::{PacketMetadata, UdpMetadata, UdpSocket}};
use embassy_time::{with_timeout, Duration};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use crate::sd_card::{SdCardSource, Storage, SdBlockDevice, SdSpiBlockingProxy};

const BLOCK_SIZE: usize = 512;
const MAX_RETRIES: u8 = 5;
const ACK_TIMEOUT: Duration = Duration::from_secs(2);

// Opcodes (RFC 1350 section 5)
const OP_RRQ: u16 = 1;
const OP_WRQ: u16 = 2;
const OP_DATA: u16 = 3;
const OP_ACK: u16 = 4;
const OP_ERROR: u16 = 5;

// Error codes (subset we actually use)
const ERR_NOT_FOUND: u16 = 1;
const ERR_ILLEGAL_OP: u16 = 4;

/// Abstraction over your storage backend. Implement this for
/// your SD-card / log-storage layer.
pub trait FileSource {
    /// Attempt to open a file by the name sent in the RRQ 
    /// Return `false` if it doesn't exist / can't be opened.
    async fn open(&mut self, filename: &str) -> bool;

    /// Read up to `buf.len()` bytes starting at byte offset `offset` from
    /// the currently open file. Return the number of bytes actually read
    /// (0 means end-of-file reached exactly at `offset`).
    async fn read_at(&mut self, offset: u32, buf: &mut [u8]) -> usize;

    /// Write every calendar day that has data into `writer`,
    /// one `YYMMDD` per line via `writer.write_date`.
    async fn list(&mut self, since: [u8; 6], writer: &mut IndexWriter<'_>);

    /// Called once when the request comes in, with the requested day's
    /// raw `"YYMMDD"` date bytes.
    /// Return `false` if there's nothing to stream (e.g. no records fall
    /// on that day) to make the server reply with a "not found" error
    /// instead of an empty file.
    async fn start_gpx(&mut self, _date: [u8; 6]) -> bool;

    /// Pulls the next chunk of already-rendered GPX bytes.
    /// Called repeatedly (once per TFTP block) after a successful
    /// `start_gpx`. Return 0 once the stream is exhausted.
    async fn next_gpx_chunk(&mut self, _buf: &mut [u8]) -> usize;
}

/// Helper passed to `FileSource::list` so implementers don't have to
/// hand-roll formatting or worry about buffer overflow.
pub struct IndexWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
    truncated: bool,
}

impl<'a> IndexWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            truncated: false,
        }
    }

    /// Appends a single `YYMMDD\r\n` line — nothing else. Silently stops
    /// writing (and marks the listing as truncated) if the buffer is
    /// full — never panics or overflows.
    pub fn write_date(&mut self, date: [u8; 6]) {
        if self.truncated {
            return;
        }
        let needed = date.len() + 2;
        if self.pos + needed >= self.buf.len() {
            self.truncated = true;
            // ...... is the same size as date
            self.buf[self.pos..self.pos + date.len()].copy_from_slice(b"......");
        } else {
            self.buf[self.pos..self.pos + date.len()].copy_from_slice(&date);
        }
        self.pos += date.len();
        self.buf[self.pos] = b'\r';
        self.buf[self.pos + 1] = b'\n';
        self.pos += 2;
    }

    fn finish(self) -> usize {
        self.pos
    }
}

/// Serves TFTP forever using the already-mounted storage. Spawn this
/// after calling `sd_card::init(...)` in `main`, passing its two return
/// values straight through:
/// ```ignore
/// let (proxy, storage) = sd_card::init(spi_bus, cs_pin).await;
/// spawner.spawn(sd_card::sd_task(proxy, storage, sd_ready)).unwrap();
/// spawner.spawn(sd_card::tftp_task(stack, storage, proxy)).unwrap();
#[embassy_executor::task]
pub async fn tftp_task(
    stack: embassy_net::Stack<'static>,
    storage: &'static Mutex<NoopRawMutex, Storage<SdBlockDevice>>,
    proxy: &'static SdSpiBlockingProxy,
) {
    let source = SdCardSource::new(storage, proxy);
    crate::tftp::serve(stack, source).await;
}

/// Runs the TFTP server loop forever. Intended to be spawned as its own
/// embassy task:
///
pub async fn serve<F: FileSource>(stack: Stack<'static>, mut source: F) -> ! {
    // to track exactly 1 incoming package
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    // TFTP ACK: 4B (opcode + block#)
    // Ethernet header: 14B, IPv4 header: 20B, UDP header: 8B
    let mut rx_buffer = [0u8; 64];

    // Exactly 1 512-byte TFTP data packet + network headers
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    // TFTP Data Payload: 512B
    // TFTP Data Header: 4B (opcode + block#)
    // Ethernet header: 14B, IPv4 header: 20B, UDP header: 8B
    let mut tx_buffer = [0u8; 576];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    let port = match env!("TFTP_PORT").parse::<u16>() {
        Ok(port) => port,
        Err(_) => 69
    };
    unwrap!(socket.bind(port),"failed to bind TFTP port");

    let mut req_buf = [0u8; 32];
    let mut data_buf = [0u8; 4 + BLOCK_SIZE]; // opcode(2) + block#(2) + data
    let mut ack_buf = [0u8; 32];
    let mut index_buf = [0u8; BLOCK_SIZE];

    loop {
        let (n, client) = match socket.recv_from(&mut req_buf).await {
            Ok(v) => v,
            Err(_) => continue,
        };

        if n < 4 {
            continue;
        }

        let opcode = u16::from_be_bytes([req_buf[0], req_buf[1]]);

        debug!("*** tftp socket.recv_from opcode {} buf {:a}", opcode, req_buf[2..n]);

        match opcode {
            OP_RRQ => {
                handle_rrq(
                    &mut socket,
                    client,
                    &req_buf[2..n],
                    &mut source,
                    &mut data_buf,
                    &mut ack_buf,
                    &mut index_buf,
                )
                .await;
            }
            OP_ACK => {}
            OP_WRQ => {
                send_error(&mut socket, client, ERR_ILLEGAL_OP, "write not supported", &mut ack_buf).await;
            }
            _ => {
                send_error(&mut socket, client, ERR_ILLEGAL_OP, "unexpected opcode", &mut ack_buf).await;
            }
        }
    }
}

/// What `send_blocks` should pull bytes from for this transfer.
enum ReadKind<'a> {
    /// Stream from the currently `open`-ed real file via `read_at`.
    RealFile,
    /// Stream from an active GPX generation session via `next_gpx_chunk`.
    Gpx,
    /// Serve a pre-rendered in-memory buffer (used for INDEX/GPXINDEX
    /// listings, already fully generated before the transfer starts).
    Buffer(&'a [u8]),
}

/// Parses the RRQ payload, then dispatches to either the virtual
/// index listing or a real file, and drives the DATA/ACK exchange
/// via `send_blocks` either way.
async fn handle_rrq<F: FileSource>(
    socket: &mut UdpSocket<'_>,
    client: UdpMetadata,
    payload: &[u8],
    source: &mut F,
    data_buf: &mut [u8],
    ack_buf: &mut [u8],
    index_buf: &mut [u8],
) {
    let Some((filename, mode)) = parse_rrq(payload) else {
        send_error(socket, client, ERR_ILLEGAL_OP, "malformed request", ack_buf).await;
        return;
    };

    if mode != "octet" {
        send_error(socket, client, ERR_ILLEGAL_OP, "only octet mode supported", ack_buf).await;
        return;
    }

    if let Some(since) = crate::gpx::parse_yymmdd(filename) {
        let mut writer = IndexWriter::new(index_buf);
        source.list(since, &mut writer).await;
        let len = writer.finish();
        send_blocks(socket, client, source, ReadKind::Buffer(&index_buf[..len]), data_buf, ack_buf).await;
        return;
    }

    if let Some(date) = crate::gpx::parse_gpx_filename(filename) {
        if !source.start_gpx(date).await {
            send_error(socket, client, ERR_NOT_FOUND, "no records for that day", ack_buf).await;
            return;
        }
        send_blocks(socket, client, source, ReadKind::Gpx, data_buf, ack_buf).await;
        return;
    }

    if !source.open(filename).await {
        send_error(socket, client, ERR_NOT_FOUND, "file not found", ack_buf).await;
        return;
    }

    send_blocks(socket, client, source, ReadKind::RealFile, data_buf, ack_buf).await;
}

/// Shared DATA/ACK block-transfer loop, used for real files, the GPX
/// stream, and pre-rendered listing buffers alike — see `ReadKind`.
async fn send_blocks<F: FileSource>(
    socket: &mut UdpSocket<'_>,
    client: UdpMetadata,
    source: &mut F,
    kind: ReadKind<'_>,
    data_buf: &mut [u8],
    ack_buf: &mut [u8],
) {
    let mut block_num: u16 = 1;
    let mut offset: u32 = 0;

    loop {
        let payload_len = match &kind {
            ReadKind::RealFile => source.read_at(offset, &mut data_buf[4..]).await,
            ReadKind::Gpx => source.next_gpx_chunk(&mut data_buf[4..]).await,
            ReadKind::Buffer(data) => {
                let start = offset as usize;
                if start >= data.len() {
                    0
                } else {
                    let cap = data_buf.len() - 4;
                    let end = (start + cap).min(data.len());
                    let n = end - start;
                    data_buf[4..4 + n].copy_from_slice(&data[start..end]);
                    n
                }
            }
        };
        let is_last = payload_len < BLOCK_SIZE;

        data_buf[0..2].copy_from_slice(&OP_DATA.to_be_bytes());
        data_buf[2..4].copy_from_slice(&block_num.to_be_bytes());
        let packet = &data_buf[..4 + payload_len];

        let mut acked = false;
        for _attempt in 0..MAX_RETRIES {
            if socket.send_to(packet, client).await.is_err() {
                continue;
            }

            match with_timeout(ACK_TIMEOUT, socket.recv_from(ack_buf)).await {
                Ok(Ok((n, from))) if from.endpoint == client.endpoint && n >= 4 => {
                    let ack_op = u16::from_be_bytes([ack_buf[0], ack_buf[1]]);
                    let ack_block = u16::from_be_bytes([ack_buf[2], ack_buf[3]]);
                    if ack_op == OP_ACK && ack_block == block_num {
                        acked = true;
                        break;
                    }
                }
                Ok(Ok(_)) => { /* not from our client, or malformed; retry */ }
                Ok(Err(_)) => { /* socket error; retry */ }
                Err(_) => { /* timed out waiting for ACK; retry send */ }
            }
        }

        if !acked {
            return; // client vanished or link dropped; abandon transfer
        }
        if is_last {
            return; // transfer complete
        }

        offset += BLOCK_SIZE as u32;
        block_num = block_num.wrapping_add(1);
    }
}

/// Parses `filename\0mode\0[option\0value\0]...` — trailing RFC 2347
/// options are ignored since we don't negotiate them.
fn parse_rrq(payload: &[u8]) -> Option<(&str, &str)> {
    let mut parts = payload.split(|&b| b == 0);
    let filename = core::str::from_utf8(parts.next()?).ok()?;
    let mode = core::str::from_utf8(parts.next()?).ok()?;
    if filename.is_empty() {
        return None;
    }
    Some((filename, mode))
}

async fn send_error(
    socket: &mut UdpSocket<'_>,
    client: UdpMetadata,
    code: u16,
    msg: &str,
    buf: &mut [u8],
) {
    buf[0..2].copy_from_slice(&OP_ERROR.to_be_bytes());
    buf[2..4].copy_from_slice(&code.to_be_bytes());
    let msg_bytes = msg.as_bytes();
    let end = 4 + msg_bytes.len().min(buf.len() - 5);
    buf[4..end].copy_from_slice(&msg_bytes[..end - 4]);
    buf[end] = 0;
    let _ = socket.send_to(&buf[..=end], client).await;
}
