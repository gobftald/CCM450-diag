use esp_hal::{
    spi::Mode,
    time::Rate,
    gpio::{Level, Output, AnyPin, OutputConfig}
};
use embassy_sync::{
    signal::Signal,
    blocking_mutex::raw::NoopRawMutex
};
use core::{
    str::from_utf8_unchecked,
    ptr::{addr_of, copy_nonoverlapping as cpn}
};

use crate::{
    SharedSpiBus,
    gps::{GpsTimeSource, GPS_DATA, GPS_UPDATED}
};

// keeping file handler and file name together
struct OpenFile<F> {
    file: Option<F>,
    file_name: [u8; 6],
}


// We use the SYNC wrapper, then we will implement 'SdSpiAdapter'
// a "Blocking-over-Async" wrapper for SD card
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;

pub struct SdSpiAdapter<T>(T);

impl<T> SdSpiAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> embedded_hal::spi::ErrorType for SdSpiAdapter<T> 
where T: embedded_hal_async::spi::SpiDevice {
    type Error = T::Error;
}

impl<T> embedded_hal::spi::SpiDevice for SdSpiAdapter<T>
where T: embedded_hal_async::spi::SpiDevice {
    fn transaction(&mut self, operations: &mut [embedded_hal::spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
        // This bridges the Async SPI device to the Blocking trait
        // Note: This is only safe because we are inside an Embassy task!
        embassy_futures::block_on(self.0.transaction(operations))
    }
}


#[embassy_executor::task]
pub(crate) async fn sd_task(
    spi_bus: &'static SharedSpiBus,
    sd_ready: &'static Signal<NoopRawMutex, ()>,
    cs_pin: AnyPin<'static>) {

    // get GPS_UPDATES watch receiver
    let mut gps_updated = GPS_UPDATED.receiver().unwrap();

    // wait for gps fix
    gps_updated.changed().await;

    let sd_config = esp_hal::spi::master::Config::default()
        // it is the highest experimental speed for ESP32-S3-Touch-LCD-2
        // and the currently inserted sd card
        .with_frequency(Rate::from_mhz(8))
        .with_mode(Mode::_0);

    let sd_card = 'init: loop {
        for attempt in 0..5 {
            let cs = Output::new(
                unsafe { cs_pin.clone_unchecked() }, Level::High,
                OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up)
            );

            let device = SpiDeviceWithConfig::new(spi_bus, cs, sd_config);
            let adapter = SdSpiAdapter::new(device);
            let card = embedded_sdmmc::SdCard::new(adapter, embassy_time::Delay);

            match card.num_bytes() {
                Ok(size) => {
                    trace!("SD init OK: {} bytes", size);
                    break 'init card;
                }
                Err(e) if attempt < 4 => {
                    embassy_time::Timer::after_millis(100).await;
                    trace!("attempt {} error {}", attempt, e);
                    continue;
                }
                Err(e) => {
                    panic!("SD init failed: {:?}", e);
                }
            }
        }
    };

    // signaling handshake finised to LCD task
    sd_ready.signal(());

    // Pass the GpsTimeSource here!
    let volume_mgr = embedded_sdmmc::VolumeManager::new(sd_card, GpsTimeSource);
    
    let volume =  unwrap!(volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0)));
    let root_dir = unwrap!(volume.open_root_dir());

    let _ = root_dir.iterate_dir(|entry| {
        use defmt::Display2Format;
        info!(
            "{} {} {} {}",
            Display2Format(&entry.name),
            entry.size,
            Display2Format(&entry.mtime),
            if entry.attributes.is_directory() {
                "<DIR>"
            } else {
                ""
            }
        );
    });

    // create the actual/initial OpenFile
    let mut current = OpenFile {
        file: None, file_name: [b' '; 6]
    };

    // get file name from date string
    unsafe {
        cpn(
        addr_of!(GPS_DATA.date) as *const u8,
        current.file_name.as_mut_ptr(),
        6
        );
    }

    // open file
    current.file = Some(
        unwrap!(
            root_dir.open_file_in_dir(
                unsafe { from_utf8_unchecked(&current.file_name[..]) },
                embedded_sdmmc::Mode::ReadWriteCreateOrAppend
            )
        )
    );

    // log gps output
    loop {
        unsafe {
            if &GPS_DATA.date[..] != &current.file_name[..] {
                // date has changed so we need to close the old file

                // don't get File than close(), it disrupts inference
                //
                // when no old file is open: None is dropped
                // when old file is open: drop will close that file
                let _old_file = current.file.take();

                // no log file is open, we should create a new one or open if it does not exist
                cpn(
                    // get file nanme from date string
                    addr_of!(GPS_DATA.date) as *const u8,
                    current.file_name.as_mut_ptr(),
                    6
                );

                current.file = Some(
                    unwrap!(
                        root_dir.open_file_in_dir(
                            from_utf8_unchecked(&current.file_name[..]),
                            embedded_sdmmc::Mode::ReadWriteCreateOrAppend
                        )
                    )
                );
            }
        }

        unsafe { debug!("{:a}", *&raw const GPS_DATA); }

        // wait for next gps update
        gps_updated.changed().await;
    }
}


/*
const SECTOR_SIZE: usize = 512;
let mut buffer = [0u8; SECTOR_SIZE];
let mut buf_idx = 0;

loop {
    delay_1_second();
    let line = format!("{}\n", get_data());
    let line_bytes = line.as_bytes();

    // If this line would overflow our 512-byte sector buffer,
    // write the full sector and continue
    if buf_idx + line_bytes.len() > SECTOR_SIZE {
        // Write the complete 512-byte sector
        my_file.write(&buffer[..buf_idx])?;  // FAT updated here
        buf_idx = 0;
        // No need to flush unless you care about directory entry size
    }

    // Copy into our sector buffer
    buffer[buf_idx..buf_idx + line_bytes.len()].copy_from_slice(line_bytes);
    buf_idx += line_bytes.len();
}
*/

/*
        let root_dir = volume.open_root_dir().unwrap();
        trace!("root_dir: {}", root_dir);

*/
