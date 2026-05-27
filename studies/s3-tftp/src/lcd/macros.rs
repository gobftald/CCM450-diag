macro_rules! lcd_init {
    ($spi_bus:ident, $cs_pin:ident, $dc_pin:ident, $reset_pin:ident, $backlight_pin:ident) => {{
        use esp_hal::{
            gpio::{Level, Output, OutputConfig},
            spi::master::Config as SpiConfig,
        };
        use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;

        let cs = Output::new($cs_pin, Level::High, OutputConfig::default());
        let dc = Output::new($dc_pin, Level::Low, OutputConfig::default());
        let reset = Output::new($reset_pin, Level::High, OutputConfig::default());
        Output::new($backlight_pin, Level::High, OutputConfig::default());

        let config = SpiConfig::default()
            //.with_frequency(esp_hal::time::Rate::from_mhz(20))
            .with_frequency(esp_hal::time::Rate::from_mhz(20))
            .with_mode(esp_hal::spi::Mode::_0);

        let device = SpiDeviceWithConfig::new($spi_bus, cs, config);
        let di = SpiInterface::new(device, dc);
        
        lcd_async::Builder::new(lcd_async::models::ST7789, di)
            .reset_pin(reset)
            .display_size(WIDTH as u16, HEIGHT as u16)
            .orientation(Orientation {
                rotation: Rotation::Deg270,
                mirrored: false,
            })
            .display_offset(0, 0)
            .invert_colors(ColorInversion::Inverted)
            .init(&mut embassy_time::Delay)
            .await
            .unwrap()            
    }};
}

macro_rules! create_no_input_pin {
    () => {
        pub struct NoInputPin;

        impl embedded_hal::digital::ErrorType for NoInputPin {
            type Error = core::convert::Infallible;
        }

        impl embedded_hal::digital::InputPin for NoInputPin {
            fn is_high(&mut self) -> Result<bool, Self::Error> { Ok(true) }
            fn is_low(&mut self) -> Result<bool, Self::Error> { Ok(false) }
        }
    }
}