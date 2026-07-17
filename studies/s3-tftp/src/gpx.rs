/// Exact size of one raw GPS log record (the on-device `GpsData` struct).
pub const RECORD_SIZE: usize = crate::sd_card::RECORD_SIZE as usize;

mod layout {
    use crate::gps::GpsData;
    use core::mem::{size_of_val, MaybeUninit};

    macro_rules! field_len {
        ($field:ident) => {
            const {
                let dummy = MaybeUninit::<GpsData>::uninit();
                unsafe { size_of_val(&(*dummy.as_ptr()).$field) }
            }
        };
    }

    pub const DATE_OFFSET: usize = 0;
    pub const DATE_LEN: usize = field_len!(date);

    pub const TIME_OFFSET: usize = DATE_OFFSET + DATE_LEN + 1;
    pub const TIME_LEN: usize = field_len!(time);

    pub const LAT_OFFSET: usize = TIME_OFFSET + TIME_LEN + 1;
    pub const LAT_LEN: usize = field_len!(lat);

    pub const LON_OFFSET: usize = LAT_OFFSET + LAT_LEN + 1;
    pub const LON_LEN: usize = field_len!(lon);

    pub const ALT_OFFSET: usize = LON_OFFSET + LON_LEN + 1;
    pub const ALT_LEN: usize = field_len!(alt);
}

/// One parsed GPS log record.
#[derive(Clone, Copy, Debug)]
pub struct RawRecord {
    pub date: [u8; 6],  // raw "YYMMDD" bytes, straight from the record
    pub time: [u8; 6],  // raw "HHMMSS" bytes, straight from the record
    pub lat: [u8; 11],  // signed decimal degrees (+N / -S)
    pub lat_len: usize,
    pub lon: [u8; 12],  // signed decimal degrees (+E / -W)
    pub lon_len: usize,
    pub alt: [u8; 4],   // meters "0000"
    pub alt_len: usize,
}

impl RawRecord {
    pub fn parse(bytes: &[u8; RECORD_SIZE]) -> Option<Self> {
        use layout as l;

        let field = |offset: usize, len: usize| -> Option<&[u8]> {
            bytes.get(offset..offset + len)
        };

        let date: [u8; 6] = field(l::DATE_OFFSET, l::DATE_LEN)?.try_into().ok()?;
        
        let time: [u8; 6] = field(l::TIME_OFFSET, l::TIME_LEN)?.try_into().ok()?;
        
        let mut lat_tmp: [u8; 12] = field(l::LAT_OFFSET, l::LAT_LEN)?.try_into().ok()?;
        let lat_len = strip_zero(&mut lat_tmp);
        let mut lat : [u8; 11] = [0; 11];
        let lat_len = nmea_to_gpx_ascii(&lat_tmp[..lat_len], &mut lat)?;
        
        let mut lon_tmp: [u8; 13] = field(l::LON_OFFSET, l::LON_LEN)?.try_into().ok()?;
        let lon_len = strip_zero(&mut lon_tmp);
        let mut lon: [u8; 12] = [0;12];
        let lon_len = nmea_to_gpx_ascii(&lon_tmp[..lon_len], &mut lon)?;
        
        let mut alt: [u8; 4] = field(l::ALT_OFFSET, l::ALT_LEN)?.try_into().ok()?;
        let alt_len = strip_zero(&mut alt);
        
        Some(RawRecord { date, time, lat, lat_len, lon, lon_len, alt, alt_len })
    }
}

pub fn nmea_to_gpx_ascii(nmea: &[u8], out_buf: &mut [u8]) -> Option<usize> {
    // identify coordinate type (Lat vs Lon) and hemisphere direction
    let suffix = nmea[nmea.len() - 1];
    let is_negative = suffix == b'S' || suffix == b'W';
    let dot_idx = nmea.iter().position(|&b| b == b'.')?;
    let deg_len  = dot_idx - 2;

    // parse digits manually from ASCII to integer (Zero-allocation parsing)
    let mut degrees: i32 = 0;
    for &b in &nmea[0..deg_len] {
        degrees = degrees * 10 + (b - b'0') as i32;
    }

    let mut min_whole: i32 = 0;
    for &b in &nmea[deg_len..dot_idx] {
        min_whole = min_whole * 10 + (b - b'0') as i32;
    }

    let mut min_frac: i32 = 0;
    for &b in &nmea[dot_idx + 1..nmea.len() - 1] {
        min_frac = min_frac * 10 + (b - b'0') as i32;
    }

    // calculate fixed-point decimal fraction via integer division
    let minutes_combined = (min_whole * 10_000_000) + min_frac * 10;
    let decimal_part = minutes_combined / 60; // Represents .0000000 to .9999999

    // serialize integers back into ASCII bytes within out_buf
    let mut cursor = 0;

    if is_negative {
        out_buf[cursor] = b'-';
        cursor += 1;
    }

    // write degrees
    cursor = write_int_to_ascii(degrees, out_buf, cursor);

    // write decimal separator point
    out_buf[cursor] = b'.';
    cursor += 1;

    // write the 7-digit decimal fraction with mandatory zero-padding
    if min_whole < 6 {
        out_buf[cursor] = b'0';
        cursor += 1;
    }
    cursor = write_int_to_ascii(decimal_part, out_buf, cursor);

    Some(cursor)
}

fn write_int_to_ascii(mut value: i32, buf: &mut [u8], start: usize) -> usize {
    let mut temp = [0u8; 10];
    let mut t_idx = 0;
    while value > 0 {
        temp[t_idx] = b'0' + (value % 10) as u8;
        value /= 10;
        t_idx += 1;
    }
    let mut cursor = start;
    while t_idx > 0 {
        t_idx -= 1;
        buf[cursor] = temp[t_idx];
        cursor += 1;
    }
    cursor
}

fn strip_zero(buf: &mut [u8]) -> usize {
    let first = buf.iter().position(|&b| b != b'0').unwrap_or(buf.len() - 1);    
    buf.copy_within(first.., 0);
    buf.len() - first
}


/// Parses an exact 6-digit calendar-day filename like `"260713.gpx"` (lowercase,
/// exact match) into the raw `"YYMMDD"` date bytes. Returns `None` if `filename`
/// doesn't match this exact pattern (six digits, then `.gpx`,nothing else)
pub fn parse_gpx_filename(filename: &str) -> Option<[u8; 6]> {
    const EXT: &str = ".gpx";
    if filename.len() != 6 + EXT.len() {
        return None;
    }
    let (digits, ext) = filename.split_at(6);
    if ext != EXT {
        return None;
    }
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(digits.as_bytes().try_into().ok()?)
}

pub const GPX_HEADER: &[u8; 374] =
b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n\
<gpx\r\n\
version=\"1.1\"\r\n\
creator=\"CCM GP450 gps-logger - https://github.com/gobftald/CCM450-diag\"\r\n\
xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"\r\n\
xmlns=\"http://www.topografix.com/GPX/1/1\"\r\n\
xsi:schemaLocation=\"http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd\">\r\n\
<trk>\r\n\
<name>YYMMDD.GPX</name>\r\n\
<trkseg>\r\n";
pub const FILE_NAME_POSITION: usize = 345;

pub const GPX_FOOTER: &[u8; 27] = b"</trkseg>\r\n</trk>\r\n</gpx>\r\n";

/// Scratch buffer size for rendering one `<trkpt>` element. Generous
/// headroom for the fields; bump it if you add more extension fields.
//const SCRATCH_SIZE: usize = 256;
// header needs bigger
const SCRATCH_SIZE: usize = 512;

fn render_gpx(record: &RawRecord, buf: &mut [u8]) -> usize {
    buf[0..12].copy_from_slice(b"<trkpt lat=\"");
    
    let eof_lat: usize = 12 + record.lat_len;
    buf[12..eof_lat].copy_from_slice(&record.lat[..record.lat_len]);
    
    let sof_lon = eof_lat + 7;
    buf[eof_lat..sof_lon].copy_from_slice(b"\" lon=\"");
    
    let eof_lon = sof_lon + record.lon_len;
    buf[sof_lon..eof_lon].copy_from_slice(&record.lon[..record.lon_len]);
    
    let sof_alt = eof_lon + 7;
    buf[eof_lon..sof_alt].copy_from_slice(b"\"><ele>");
    
    let eof_alt = sof_alt + record.alt_len;
    buf[sof_alt..eof_alt].copy_from_slice(&record.alt[..record.alt_len]);
    
    let sof_time = eof_alt + 16;
    buf[eof_alt..sof_time].copy_from_slice(b"</ele>\r\n<time>20");
    
    buf[sof_time] = record.date[0];
    buf[sof_time + 1] = record.date[1];
    buf[sof_time + 2] = b'-';
    buf[sof_time + 3] = record.date[2];
    buf[sof_time + 4] = record.date[3];
    buf[sof_time + 5] = b'-';
    buf[sof_time + 6] = record.date[4];
    buf[sof_time + 7] = record.date[5];
    buf[sof_time + 8] = b'T';
    
    buf[sof_time + 9] = record.time[0];
    buf[sof_time + 10] = record.time[1];
    buf[sof_time + 11] = b':';
    buf[sof_time + 12] = record.time[2];
    buf[sof_time + 13] = record.time[3];
    buf[sof_time + 14] = b':';
    buf[sof_time + 15] = record.time[4];
    buf[sof_time + 16] = record.time[5];
    
    let eof_time = sof_time + 17;
    buf[eof_time..eof_time + 19].copy_from_slice(b"Z</time></trkpt>\r\n\n");

    eof_time + 19
}

/// Supplies raw records for GPX streaming for a single calendar day,
/// identified by its raw `"YYMMDD"` date bytes
/// Implement this for your SD card layer: it should internally track
/// which sector/file it's at across repeated calls, advancing in timestamp
/// order, and return `None` once nothing more matching `date` remains.
pub trait RecordSource {
    async fn next_record(&mut self, date: [u8; 6]) -> Option<RawRecord>;
}

enum GpxState {
    Header,
    Points,
    Footer,
    Done,
}

/// Streaming state machine: call `next_chunk` repeatedly (once per TFTP
/// block) to pull successive slices of GPX text. Holds only one record's
/// worth of scratch space regardless of how many points the track has.
pub struct GpxStreamer {
    state: GpxState,
    date: [u8; 6],
    scratch: [u8; SCRATCH_SIZE],
    scratch_len: usize,
    scratch_pos: usize,
}

impl GpxStreamer {
    pub fn new(date: [u8; 6]) -> Self {
        Self {
            state: GpxState::Header,
            date,
            scratch: [0; SCRATCH_SIZE],
            scratch_len: 0,
            scratch_pos: 0,
        }
    }

    /// Fills `out` with the next chunk of GPX bytes, pulling more records
    /// from `source` as needed. Returns bytes written; 0 means the stream
    /// is exhausted (GPX footer already fully flushed on a previous call).
    pub async fn next_chunk<S: RecordSource>(&mut self, source: &mut S, out: &mut [u8]) -> usize {
        let mut written = 0;
        while written < out.len() {
            if self.scratch_pos < self.scratch_len {
                let n = (self.scratch_len - self.scratch_pos).min(out.len() - written);
                out[written..written + n]
                    .copy_from_slice(&self.scratch[self.scratch_pos..self.scratch_pos + n]);
                written += n;
                self.scratch_pos += n;
                continue;
            }

            match self.state {
                GpxState::Header => {
                    self.scratch[..FILE_NAME_POSITION]
                        .copy_from_slice(&GPX_HEADER[..FILE_NAME_POSITION]);
                    self.scratch[FILE_NAME_POSITION..FILE_NAME_POSITION + 6]
                        .copy_from_slice(&self.date);
                    self.scratch[FILE_NAME_POSITION + 6..FILE_NAME_POSITION + 10]
                        .copy_from_slice(b".GPX");
                    self.scratch[FILE_NAME_POSITION + 10..GPX_HEADER.len()]
                        .copy_from_slice(&GPX_HEADER[FILE_NAME_POSITION + 10..GPX_HEADER.len()]);
                    self.scratch_len = GPX_HEADER.len();
                    self.scratch_pos = 0;
                    self.state = GpxState::Points;
                }
                GpxState::Points => match source.next_record(self.date).await {
                    Some(rec) => {
                        let n = render_gpx(&rec, &mut self.scratch);
                        self.scratch_len = n;
                        self.scratch_pos = 0;
                    }
                    None => self.state = GpxState::Footer,
                },
                GpxState::Footer => {
                    self.scratch[..27].copy_from_slice(GPX_FOOTER);
                    self.scratch_len = 27;
                    self.scratch_pos = 0;
                    self.state = GpxState::Done;
                }
                GpxState::Done => break,
            }
        }
        written
    }
}