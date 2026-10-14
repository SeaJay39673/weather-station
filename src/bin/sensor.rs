#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Flex;

pub async fn read_sensor_data(mut flex_pin: Flex<'static>) -> ! {
    flex_pin.apply_output_config(
        &esp_hal::gpio::OutputConfig::default()
            .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
            .with_pull(esp_hal::gpio::Pull::None),
    );
    flex_pin.set_output_enable(true);
    flex_pin.set_input_enable(true);
    flex_pin.set_high();

    let mut dht22 = embedded_dht_rs::dht22::Dht22::new(flex_pin, esp_hal::delay::Delay::new());

    loop {
        match dht22.read() {
            Ok(sensor_reading) => {
                info!("Temp: {:?}F", (&sensor_reading.temperature * (9.0 / 5.0)) + 32.0);
                info!("Humidity: {:?}%", &sensor_reading.humidity);
            },
            Err(e) => {
                let e = defmt::Debug2Format(&e);
                error!("{:?}", e);
            }
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}