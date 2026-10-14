#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{info};
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::gpio::Flex;
use esp_println as _;
use esp_radio::ble::controller::BleConnector;

extern crate alloc;

mod sensor;
mod bluetooth;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32s3 -o esp32s3-wroom-2 -o unstable-hal -o embassy -o alloc -o wifi -o ble-trouble -o stack-smashing-protection -o defmt -o esp-backtrace -o vscode -o esp

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO0
    // - GPIO3
    // - GPIO45
    // - GPIO46
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _gpio33 = peripherals.GPIO33;
    let _gpio34 = peripherals.GPIO34;
    let _gpio35 = peripherals.GPIO35;
    let _gpio36 = peripherals.GPIO36;
    let _gpio37 = peripherals.GPIO37;

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");

    // let _wifi_controller =
    //     esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default())
    //         .expect("Failed to initialize Wi-Fi controller");
    // let _wifi_interface = esp_radio::wifi::Interface::station();
    // // find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
    let transport = match BleConnector::new(peripherals.BT, Default::default()) {
        Ok(transport) => transport,
        Err(e) => panic!("{:?}", e),
    };

    let bluetooth_spawn_token = match bluetooth::run_bluetooth(transport) {
        Ok(spawn_token) => spawn_token,
        Err(e) => panic!("{:?}", e)
    };
    
    spawner.spawn(bluetooth_spawn_token);

    sensor::read_sensor_data(Flex::new(peripherals.GPIO4)).await
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}