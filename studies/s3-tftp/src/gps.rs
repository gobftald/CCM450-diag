use esp_hal::{
    Async,
    gpio::AnyPin,
    uart::{AnyUart, Config, RxConfig, RxError, Uart, UartRx, UartTx},
};
use core::slice::from_raw_parts;

#[macro_export]
macro_rules! create_gps_uart {
    ($peripherals:ident) => {
        crate::gps::GpsUart::new(
            $peripherals.UART0.into(),
            $peripherals.GPIO43.into(),
            $peripherals.GPIO44.into(),
        )
    }
}

pub struct GpsUart<'a> {
    pub(crate) rx: UartRx<'a, Async>,
    pub(crate) tx: UartTx<'a, Async>,
}

impl<'a> GpsUart<'a> {
    pub fn new(uart: AnyUart<'static>, tx_pin: AnyPin<'static>, rx_pin: AnyPin<'static>) -> Self {
        let config = Config::default()
            //.with_baudrate(115_200)
            .with_baudrate(9_600)
            .with_rx(
                // explicitely disable timeout
                RxConfig::default().with_timeout_none()
            );
        let mut uart = unwrap!(Uart::new(uart, config))
            .into_async()
            .with_tx(tx_pin)
            .with_rx(rx_pin);
        uart.set_at_cmd(esp_hal::uart::AtCmdConfig::default()
            .with_pre_idle_count(0)
            .with_post_idle_count(0)
            .with_cmd_char(b'\n'));
        let (rx, tx) = uart.split();

        Self { rx, tx }
    }
}

use core::marker::PhantomData;
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
pub static GPS_UPDATED: Watch<SyncNoopRawMutex, (), 3> = Watch::new();

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C, packed)]  // Remove all padding
pub struct GpsData {
    pub date: [u8; 6],
    sep0: u8,
    pub time: [u8; 6],
    sep1: u8,
    pub lat: [u8; 12],
    sep2: u8,
    pub lon: [u8; 13],
    sep3: u8,
    pub alt: [u8; 4],
    sep4: u8,
    pub sog: [u8; 3],
    sep5: u8,
    pub sat: [u8; 2],
    sep6: u8,
    pub hdop: [u8;5],
    sep7: u8,
    pub cog: [u8; 3],
}

pub static mut GPS_DATA: GpsData = GpsData {
    date: *b"yymmdd",
    sep0: b',',
    time: *b"hhmmss",
    sep1: b',',
    lat: *b"0000.000000N",
    sep2: b',',
    lon: *b"00000.000000E",
    sep3: b',',
    alt: *b"0000",
    sep4: b',',
    sog: *b"000",
    sep5: b',',
    sat: *b"00",
    sep6: b',',
    hdop: *b"00.00",
    sep7: b',',
    cog: *b"000",
};

#[embassy_executor::task()]
pub async fn gps_task(mut gps: crate::gps::GpsUart<'static>) {
    use core::ptr::{addr_of_mut, copy_nonoverlapping as cpn};

    let mut buf = [0u8; 128];
    let mut fix: usize = 0;             // gps fix status
    let mut ofx: usize = 0;             // overflow index

    let mut update: bool = false;       // send update signal
    let gps_updated = GPS_UPDATED.sender();

    //let mut tc0 = 0;

    loop {
        // We need more than minimum = 1, to get enough time for all operations below
        // before the next wait_for_buffered_data.
        // read_async call wait_for_buffered_data with a hardwired minimum = 1.
        //
        // However, if a wait_for_buffered_data was woken much later then "IRQ at_cmd_char_det",
        // e.g. because of a lenghty lcd refresh, or bacause of any other future situation,
        // then even we called read_buffered hence clearing the fifo, then even after the short
        // processing time of operations below (max < 200us), the next wait_for_buffered_data
        // found a new bytes in fifo, collapsing the whole overflow management.
        //
        // It is why we need to make wait_for_buffered_data public (minimum = 2)
        /*
        let tc1 = xtensa_lx::timer::get_cycle_count();
        debug!("wait again {}us", (tc1 - tc0) / 240);
        tc0 = tc1;
        */
        match gps.rx.wait_for_buffered_data(2, buf.len(), false).await {
            Ok(_) => {
                //tc0 = xtensa_lx::timer::get_cycle_count();

                // if "read" gives error, it will be "handled" in the next loop
                // if we've used wait_for_buffered_data we can call read_buffered directly
                //if let Ok(size) = gps.rx.read_async(&mut buf[ofx..]).await {
                if let Ok(size) = gps.rx.read_buffered(&mut buf[ofx..]) {
                    debug!("got size {} {:a}", size, &buf[..size + ofx]);

                    match &buf[1..6] {
                        b"GNGGA" => {
                            if let Ok(fld) = field(6, &buf[..size]) {
                                if fld.len() > 0 {
                                    if fld[0] == b'0' {
                                        fix = 0;
                                    } else {
                                        fix += 1;
                                    }
                                }
                            }

                            if fix > 1 {
                                if let Ok(fld) = field(2, &buf[..size]) {
                                    if fld.len() == 11 {
                                        unsafe {
                                            cpn(
                                                fld.as_ptr(),
                                                addr_of_mut!((*(&raw mut GPS_DATA)).lat) as *mut u8,
                                                11
                                            );
                                        }
                                    }
                                }

                                if let Ok(fld) = field(3, &buf[..size]) {
                                    if fld.len() == 1 {
                                        unsafe { *&raw mut GPS_DATA.lat[11] = fld[0]; }
                                    }
                                }

                                if let Ok(fld) = field(4, &buf[..size]) {
                                    if fld.len() == 12 {
                                        unsafe {
                                            cpn(
                                                fld.as_ptr(),
                                                addr_of_mut!((*(&raw mut GPS_DATA)).lon) as *mut u8,
                                                12
                                            );
                                        }
                                    }
                                }

                                if let Ok(fld) = field(5, &buf[..size]) {
                                    if fld.len() == 1 {
                                        unsafe { *&raw mut GPS_DATA.lon[12] = fld[0]; }
                                    }
                                }

                                if let Ok(fld) = field(7, &buf[..size]) {
                                    if fld.len() == 2 {
                                        unsafe {
                                            cpn(
                                                fld.as_ptr(),
                                                addr_of_mut!((*(&raw mut GPS_DATA)).sat) as *mut u8,
                                                2
                                            );
                                        }
                                    }
                                }

                                if let Ok(fld) = field(8, &buf[..size]) {
                                    if fld.len() > 0 {
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
                                    }
                                }

                                if let Ok(fld) = field(9, &buf[..size]) {
                                    if fld.len() > 0 {
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
                            }
                        }

                        b"GNRMC" => {
                            if fix > 1 {
                                if let Ok(fld)= field(1, &buf[..size]) {
                                    if fld.len() == 10 {
                                        unsafe {
                                            cpn(
                                                fld.as_ptr(),
                                                addr_of_mut!((*(&raw mut GPS_DATA)).time) as *mut u8,
                                                6
                                            );

                                            // update if we have got at least a new timestamp
                                            update = true;
                                        }
                                    }
                                }

                                if let Ok(fld) = field(9, &buf[..size]) {
                                    if fld.len() == 6 {
                                        let mut swap: [u8; 6] = [0; 6];
                                        swap[0] = fld[4]; swap[1] = fld[5];
                                        swap[2] = fld[2]; swap[3] = fld[3];
                                        swap[4] = fld[0]; swap[5] = fld[1];

                                        unsafe {
                                            cpn(
                                                swap.as_ptr(),
                                                addr_of_mut!((*(&raw mut GPS_DATA)).date) as *mut u8,
                                                6
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        b"GNVTG" => {
                            if fix > 1 {
                                if let Ok(fld)= field(1, &buf[..size]) {
                                    if fld.len() == 6 {
                                        unsafe {
                                            cpn(
                                                fld.as_ptr(),
                                                addr_of_mut!((*(&raw mut GPS_DATA)).cog) as *mut u8,
                                                3
                                            );
                                        }
                                    }
                                }

                                if let Ok(fld) = field(7, &buf[..size]){
                                    if fld.len() > 0 {
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
                                    }
                                }

                                // send update signal only from the 3rd sentence
                                if update {
                                    //unsafe { debug!("{:a}", *&raw const GPS_DATA); }
                                    //debug!("gps_updated.send");

                                    // Signal that GPS_DATA were updated 
                                    gps_updated.send(());

                                    update = false;
                                }
                            }
                        }

                        // drop all other sentence (like. PAIR)
                        _ => {}
                    }

                    // handle overflow
                    if buf[size + ofx - 1] != b'\n' {
                        if let Some(i) =  buf[..size + ofx].iter().rev().position(|c| *c == b'$' ) {
                            ofx = i + 1;
                            unsafe {
                                cpn(
                                    buf.as_ptr().add(size - ofx),
                                    buf.as_mut_ptr().add(0),
                                    ofx
                                );
                            }
                        } else {
                            ofx = 0;
                        }
                    } else {
                        ofx = 0;
                    }
                }
            },
            Err(error) => {
                debug!("RxError {}", error);

                match error {
                    // most likely after boot
                    RxError::FifoOverflowed => {
                        // this is the only way to call rxfifo_reset()
                        let _ = gps.rx.check_for_errors();
                    },
                    _ => {
                        // skip at least the next gps timestamp
                        embassy_time::Timer::after_millis(1_500).await;
                    },
                }
            }
        }
    }
}

fn field(mut field_num: usize, buf: &[u8]) -> Result<&[u8], ()> {
    let mut rbuf = buf;
    let mut len: i32 = buf.len() as i32;

    while field_num > 0 {
        for (i, p) in rbuf.iter().enumerate() {
            if *p == b',' {
                len -= i as i32  + 1;
                if len > 0 {
                    unsafe { rbuf = from_raw_parts((p as *const u8).add(1) , len as usize); }
                    break
                } else {
                    return Err(());
                }
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
        return Err(());
    };

    Ok(&rbuf[..size])
}
