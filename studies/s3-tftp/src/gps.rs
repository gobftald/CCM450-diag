use esp_hal::{
    Async,
    gpio::AnyPin,
    uart::{AnyUart, Config, Uart, UartRx, UartTx},
};

#[macro_export]
macro_rules! create_gps {
    ($peripherals:ident) => {
        crate::gps::GPS::new(
            $peripherals.UART0.into(),
            $peripherals.GPIO43.into(),
            $peripherals.GPIO44.into(),
        )
    }
}

pub struct GPS<'a> {
    pub(crate) rx: UartRx<'a, Async>,
    pub(crate) tx: UartTx<'a, Async>,
}

impl<'a> GPS<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        // configure UART
        let config = Config::default().with_baudrate(115_200);
        let mut uart = unwrap!(Uart::new(uart, config))
            .into_async()
            .with_tx(tx_pin)
            .with_rx(rx_pin);
        uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default()
            .with_pre_idle_count(0)
            .with_post_idle_count(0)
            .with_gap_timeout(20)
            .with_cmd_char(b'\n'));
        let (rx, tx) = uart.split();

        Self { rx, tx }
    }
}

use core::marker::{self, PhantomData};
use embassy_sync::{blocking_mutex::raw::RawMutex, watch::Watch};

// Custom mutex that is explicitly Sync for Signal
pub struct SyncNoopRawMutex {
    _phantom: PhantomData<*mut ()>,
}

// SAFETY: We are using this only on a single-core, single-executor system
unsafe impl Send for SyncNoopRawMutex {}
unsafe impl Sync for SyncNoopRawMutex {}

impl SyncNoopRawMutex {
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

unsafe impl RawMutex for SyncNoopRawMutex {
    const INIT: Self = Self::new();

    fn lock<R>(&self, f: impl FnOnce() -> R) -> R {
        // No actual locking - we're single-threaded
        f()
    }
}

// Signal to sd_card and bluetooth
pub static GPS_UPDATED: Watch<SyncNoopRawMutex, (), 2> = Watch::new();

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed)]  // Remove all padding
pub struct GpsData {
    pub date: [u8; 6],
    pub time: [u8; 6],
    pub lat: [u8; 12],
    pub lon: [u8; 13],
    pub sat: [u8; 2],
    pub hdop: [u8;5],
    pub alt: [u8; 4],
    pub cog: [u8; 3],
    pub sog: [u8; 3],
}

pub static mut GPS_DATA: GpsData = GpsData {
    date: *b"yymmdd",
    time: *b"hhmmss",
    lat: *b"0000.000000N",
    lon: *b"00000.000000E",
    sat: *b"00",
    hdop: *b"00.00",
    alt: *b"0000",
    cog: *b"000",
    sog: *b"000",
};

#[embassy_executor::task()]
pub async fn gps_task(mut gps: crate::gps::GPS<'static>) {
    use core::ptr::{addr_of_mut, copy_nonoverlapping as cpn};

    let mut buf = [0u8; 128];
    let mut fix: usize = 0;
    let gps_updated = GPS_UPDATED.sender();

    loop {
        let match_buf: &[u8];
        // wait for incoming NMEA messages but ignore RxError
        // we should made this fn public to receive NMEA messages correctly
        // minimum comes (below) from the shortest message size ($GNVTG before satelite fix)
        let _ = gps.rx.wait_for_buffered_data(20, buf.len(), true).await;

        if let Ok(size) =  gps.rx.read_buffered(&mut buf) {
            //debug!("{:a}", &buf[..size]);

            // sometime the first '$' has already read after the last '\n'
            if buf[0] == b'$' {
                match_buf = &buf[1..6]
            } else {
                match_buf = &buf[..5];
            }

            match match_buf {
                b"GNGGA" => {
                    let mut fld= field(6, &buf[..size]);
                    if fld[0] == b'0' {
                        fix = 0;
                    } else {
                        fix += 1;
                    }

                    if fix > 1 {
                        fld= field(2, &buf[..size]);
                        if fld.len() == 11 {
                            unsafe {
                                cpn(
                                    fld.as_ptr(),
                                    addr_of_mut!((*(&raw mut GPS_DATA)).lat) as *mut u8,
                                    11
                                );
                            }
                        }

                        fld = field(3, &buf[..size]);
                        if fld.len() == 1 {
                            unsafe { *&raw mut GPS_DATA.lat[11] = fld[0]; }
                        }

                        fld = field(4, &buf[..size]);
                        if fld.len() == 12 {
                            unsafe {
                                cpn(
                                    fld.as_ptr(),
                                    addr_of_mut!((*(&raw mut GPS_DATA)).lon) as *mut u8,
                                    12
                                );
                            }
                        }

                        fld = field(5, &buf[..size]);
                        if fld.len() == 1 {
                            unsafe { *&raw mut GPS_DATA.lon[12] = fld[0]; }
                        }

                        fld = field(7, &buf[..size]);
                        if fld.len() == 2 {
                            unsafe {
                                cpn(
                                    fld.as_ptr(),
                                    addr_of_mut!((*(&raw mut GPS_DATA)).sat) as *mut u8,
                                    2
                                );
                            }
                        }

                        fld = field(8, &buf[..size]);
                        let mut len = fld.len();
                        unsafe {
                            for p in (*&raw mut GPS_DATA.hdop).iter_mut().rev() {
                                if len != 0 {
                                    *p = fld[len - 1];
                                    len -= 1;
                                } else {
                                    *p = b'0';
                                }
                            }
                        }

                        fld = field(9, &buf[..size]);
                        for (mut i, p) in fld.iter().enumerate() {
                            if *p == b'.' {
                                unsafe {
                                    for p in (*&raw mut GPS_DATA.alt).iter_mut().rev() {
                                        if i > 0 {
                                            *p = fld[i - 1];
                                            i -= 1;
                                        } else {
                                            *p = b'0';
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }

                b"GNRMC" => {
                    if fix > 1 {
                        let mut fld= field(1, &buf[..size]);
                        if fld.len() == 10 {
                            unsafe {
                                cpn(
                                    fld.as_ptr(),
                                    addr_of_mut!((*(&raw mut GPS_DATA)).time) as *mut u8,
                                    6
                                );
                                GPS_TIMESTAMP.hours = (fld[0] - b'0') * 10 + (fld[1] - b'0');
                                GPS_TIMESTAMP.minutes = (fld[2] - b'0') * 10 + (fld[3] - b'0');
                                GPS_TIMESTAMP.seconds = (fld[4] - b'0') * 10 + (fld[5] - b'0');
                            }
                        }

                        fld = field(9, &buf[..size]);

                        let mut swap: [u8; 6] = [0; 6];
                        swap[0] = fld[4]; swap[1] = fld[5];
                        swap[2] = fld[2]; swap[3] = fld[3];
                        swap[4] = fld[0]; swap[5] = fld[1];

                        if fld.len() == 6 {
                            unsafe {
                                cpn(
                                    swap.as_ptr(),
                                    addr_of_mut!((*(&raw mut GPS_DATA)).date) as *mut u8,
                                    6
                                );
                                GPS_TIMESTAMP.year_since_1970 = (swap[0] - b'0') * 10 + (swap[1] - b'0') + 30;
                                GPS_TIMESTAMP.zero_indexed_month = (swap[2] - b'0') * 10 + (swap[3] - b'0') - 1;
                                GPS_TIMESTAMP.zero_indexed_day = (swap[4] - b'0') * 10 + (swap[5] - b'0') - 1;
                            }
                        }
                    }
                }

                b"GNVTG" => {
                    if fix > 1 {
                        let mut fld= field(1, &buf[..size]);
                        if fld.len() == 6 {
                            unsafe {
                                cpn(
                                    fld.as_ptr(),
                                    addr_of_mut!((*(&raw mut GPS_DATA)).cog) as *mut u8,
                                    3
                                );
                            }
                        }

                        fld = field(7, &buf[..size]);
                        for (mut i, p) in fld.iter().enumerate() {
                            if *p == b'.' {
                                unsafe {
                                    for p in (*&raw mut GPS_DATA.sog).iter_mut().rev() {
                                        if i > 0 {
                                            *p = fld[i - 1];
                                            i -= 1;
                                        } else {
                                            *p = b'0';
                                        }
                                    }
                                }
                                break;
                            }
                        }

                        //unsafe { debug!("{}", *&raw const GPS_TIMESTAMP); }
                        //unsafe { debug!("{:a}", *&raw const GPS_DATA); }

                        // Signal that GPS_DATA and GPS_TIMESTAMP were updated 
                        gps_updated.send(());
                    }
                }

                _ => {}
            }
        }
    }
    // GGA: Lat, Lon, Sat, Alt
    // RMC: Date, Time
    // VTG: COG, SOG
}

fn field(mut field_num: usize, buf: &[u8]) -> &[u8]{
    use core::slice::from_raw_parts;

    let mut rbuf = buf;
    let mut len = buf.len();

    while field_num > 0 {
        for (i, p) in rbuf.iter().enumerate() {
            if *p == b',' {
                len -= i + 1;
                unsafe { rbuf = from_raw_parts((p as *const u8).add(1) , len); }
                break
            } else {
                continue
            }
        }
        field_num -= 1;
    }

    let size = 'block: {
        for (i, p) in rbuf.iter().enumerate() {
            if *p == b',' {
                break 'block i;
            } else {
                continue
            }
        }
        0
    };

    &rbuf[..size]
}


// sd_card needs this TimeSource stuff
use embedded_sdmmc::{Timestamp, TimeSource};

static mut GPS_TIMESTAMP: Timestamp = Timestamp {
    year_since_1970: 56, zero_indexed_month: 0, zero_indexed_day: 1, hours: 0, minutes: 0, seconds: 0 
};

pub struct GpsTimeSource;

impl TimeSource for GpsTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        // don't care if not precise timestamp caused by potential race condition
        unsafe { GPS_TIMESTAMP }
    }
}
