use esp_hal::gpio::{Level, Output, AnyPin, OutputConfig};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use crate::{SharedSpiBus, gps::GpsTimeSource};

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

    // SCOPED HANDSHAKE (400kHz)
    {    
        let cs = Output::new(
            unsafe { cs_pin.clone_unchecked() }, Level::High,
            OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up)
        );
        let sd_config = esp_hal::spi::master::Config::default()
            .with_frequency(esp_hal::time::Rate::from_khz(400));

        // Use SpiDeviceWithConfig to allow bus sharing with different settings
        // and with CS management
        let sd_device = SpiDeviceWithConfig::new(spi_bus, cs, sd_config);

        // Wrap device in our blocing adapter
        let sd_adapter = SdSpiAdapter::new(sd_device);
    
        // Dealy is a blocking call
        let sd_card = embedded_sdmmc::SdCard::new(sd_adapter, embassy_time::Delay);

        // Trigger the SPI Mode handshake before the LCD task starts
        if let Ok(size) = sd_card.num_bytes() {
            trace!("SD Handshake success: {} bytes", size);
        } else {
            error!("SD Handshake failed!");
            return;
        }
        // At the end of this block, sd_card, adapter, device, and cs are all DROPPED.
        // The pins are now free to be used again.
    }

    // 2. HIGH-SPEED SETUP (12MHz)
    let cs = Output::new(
        cs_pin, Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up)
    );
    let sd_config = esp_hal::spi::master::Config::default()
        .with_frequency(esp_hal::time::Rate::from_mhz(12));


    let device = SpiDeviceWithConfig::new(spi_bus, cs, sd_config);
    let adapter = SdSpiAdapter::new(device);
    let sd_card = embedded_sdmmc::SdCard::new(adapter, embassy_time::Delay);
    
    // signaling handshake finised to LCD task
    sd_ready.signal(());

    /*
    A Note on Speeding Up the SD Card: Once the SD card finishes its initial handshake (after the call to 
    sdcard.num_bytes()), you can technically increase its speed to 12MHz or 25MHz. However, since you are 
    prioritizing other communications, like GPS or Wifi, keeping the SD card at a lower speed is actually 
    safer, as it results in shorter, less demanding SPI bursts that are less likely to block your other tasks.

    If you hammer the card at 25MHz, the card’s internal controller might get "overwhelmed" or run into more 
    errors, leading to longer internal busy-wait periods.
    At lower speeds, the communication is more synchronous with the card's internal processing, often leading 
    to more predictable "Ready" signals.
    
    12MHz can be sweet spot.
    */ 
    
    // Pass the GpsTimeSource here!
    let mut volume_mgr = embedded_sdmmc::VolumeManager::new(sd_card, GpsTimeSource);
    
    /*
    // Now, every file created or modified will call GpsTimeSource::get_timestamp()
    if let Ok(mut volume) = volume_mgr.open_volume(embedded_sdmmc::VolumeIdx(0)) {
        let mut root_dir = volume.open_root_dir().unwrap();
        let mut file = root_dir.open_file_in_dir("log.txt", embedded_sdmmc::Mode::ReadWriteCreateOrAppend).unwrap();
        file.write(b"Data logged with GPS time!").unwrap();
    }

    // Try to get card size (forces initialization)
    loop {
        match sd_card.num_bytes() {
            Ok(size) => {
                esp_println::println!("SD Card found! Size: {} bytes", size);
                break;
            }
            Err(_) => {
                esp_println::println!("Waiting for SD card...");
                Timer::after_secs(2).await;
            }
        }
    }
    */
}
