use esp_hal::{gpio::Input, i2c::master::Error};
use embassy_time::Timer;

const QMI8658_ADDR: u8 = 0x6B;  // SDO/SA0 connected to GND
const REG_CTRL1: u8 = 0x02;     // Control power states
const REG_CTRL2: u8 = 0x03;     // Acelerometer settings (Output Data Rate, Full Scale, Self Test)
const REG_CTRL7: u8 = 0x08;     // Enable sensors
const REG_CTRL9: u8 = 0x0a;     // Host Commands
const REG_STATUSINT: u8 = 0x2d; // Sensor Data Availability
const REG_STATUS1: u8 = 0x2f;   // Miscellaneous Status: Wake on Motion

// WoM specific internal calibration registers (for CTRL9 commands)
const REG_CAL1_L: u8 = 0x0b;
const REG_CAL1_H: u8 = 0x0c;

#[embassy_executor::task]
pub(crate) async fn imu_task(
    mut i2c: esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>,
    mut imu_int: Input<'static>
) {
    let _ = imu_init(&mut i2c);

    loop {
        imu_int.wait_for_rising_edge().await;

        info!("*** imu Motion detected!");

        // Clear interrupt inside QMI8658 chip
        // Reading STATUS1 clears WoM flag and reset INT pin
        let mut status = [0u8; 1];
        let _ = i2c.write_read(QMI8658_ADDR, &[REG_STATUS1], &mut status);

        Timer::after_millis(200).await;
    }
}

fn imu_init(i2c: &mut esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>) -> Result<(), Error> {
    info!("*** imu QMI8658 WoM initialisation ...");

    // Sensors off
    i2c.write(QMI8658_ADDR, &[REG_CTRL7, 0x00])?;

    // INT1 pin output enabled
    i2c.write(QMI8658_ADDR, &[REG_CTRL1, 0x08])?;

    // Accel ODR low consumption mode (21 Hz, Low Power, aFS=±2g)
    i2c.write(QMI8658_ADDR, &[REG_CTRL2, 0b0000_1101])?;        // = 0x0D

    i2c.write(QMI8658_ADDR, &[REG_CAL1_L, 40])?;                // 40 mg threshold
    i2c.write(QMI8658_ADDR, &[REG_CAL1_H, 0b001_0101])?;       // INT1, initial level 0,
                                                                                // blanking= 1s * 21Hz = 0x15

    // sending CTRL9 "write WOM setting" command + handshake
    i2c.write(QMI8658_ADDR, &[REG_CTRL9, 0x08])?;
    loop {
        let mut status = [0u8; 1];
        i2c.write_read(QMI8658_ADDR, &[REG_STATUSINT], &mut status)?;
        if status[0] & 0x80 != 0 { break; } // CmdDone
    }
    // reading STATUSINT clears CTRL9's CmdDone bit automatically

    // Gyroscope off (Bit 1 = 0)
    // Acelerometer on (Bit 0 = 1) - in WoM mode, only Accel is allowed to go
    i2c.write(QMI8658_ADDR, &[REG_CTRL7, 0x01])?;

    Ok(())
}