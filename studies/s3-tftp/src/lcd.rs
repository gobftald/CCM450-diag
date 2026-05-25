use esp_hal::gpio::{Level, Output, AnyPin, OutputConfig};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use crate::SharedSpiBus;

#[embassy_executor::task]
pub(crate) async fn lcd_task(
    spi_bus: &'static SharedSpiBus,
    sd_ready: &'static Signal<NoopRawMutex, ()>,
    cs_pin: AnyPin<'static>,
    dc_pin: AnyPin<'static>
) {
    // WAIT for the SD card to finish its sensitive handshake
    sd_ready.wait().await;

    // Create the async device
    let cs = Output::new(
        cs_pin, Level::High,
        OutputConfig::default().with_pull(esp_hal::gpio::Pull::Up)
    );

    /* By making SdSpiAdapter concrete over SpiDmaBus rather than generic over BUS, you sidestep the 
       SetConfig trait bound issue entirely and can call apply_config directly. The LCD task does the 
       same — applies its own config after locking the mutex.
    */
    
    /*
    let dc = Output::new(dc_pin, Level::Low, OutputConfig::default());

    // Create the Config for the LCD (High Speed 20MHz)
    let lcd_config = Config::default()
        .with_frequency(esp_hal::time::Rate::from_mhz(20))
        .with_mode(SpiMode::Mode0);

    // Use SpiDeviceWithConfig to allow bus sharing with different settings
    // and with CS management
    let lcd_device = SpiDeviceWithConfig::new(spi_bus, lcd_cs, lcd_config);

    // 2. Init the driver
    let di = display_interface_spi::SPIInterfaceNoCS::new(device, dc);

    let mut display = mipidsi::Builder::st7789(di)
        .init(&mut Delay, None).await.unwrap();


    ????
    // After init, you can change the bus speed inside a lock
    {
        let mut bus_lock = bus.lock().await;
        bus_lock.apply_config(&esp_hal::spi::master::Config::default()
            .with_frequency(esp_hal::time::Rate::from_khz(20.MHz());
    }

    // 3. LVGL Integration
    // You will wrap 'display' in an LVGL "display driver"
    // Using a crate like 'lvgl' or doing raw FFI

    loop {
        display.clear(embedded_graphics::pixelcolor::Rgb565::RED).unwrap();
        Timer::after_secs(1).await;
        display.clear(embedded_graphics::pixelcolor::Rgb565::BLUE).unwrap();
        Timer::after_secs(1).await;
    }
    */
}
