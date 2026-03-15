use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use log::{info, error};

mod mhz19;
use mhz19::MHZ19;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting...");

    let peripherals = Peripherals::take().unwrap();

    // LED 闪烁线程
    let led_pin = peripherals.pins.gpio48;
    std::thread::spawn(move || {
        if let Ok(mut led) = PinDriver::output(led_pin) {
            loop {
                let _ = led.set_high();
                std::thread::sleep(std::time::Duration::from_secs(1));
                let _ = led.set_low();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    });

    // 尝试 UART2 (GPIO1 TX, GPIO3 RX)
    // 或者用其他空闲引脚
    info!("Trying UART2 on GPIO1/GPIO3...");
    let uart2 = peripherals.uart2;
    let tx = peripherals.pins.gpio1;
    let rx = peripherals.pins.gpio2;

    match MHZ19::new(uart2, tx, rx) {
        Ok(mut sensor) => {
            info!("Sensor initialized on UART2");
            esp_idf_svc::hal::task::block_on(async {
                loop {
                    match sensor.read_co2().await {
                        Ok(co2) => info!("CO2: {}", co2),
                        Err(e) => error!("ERR: {}", e),
                    }
                    FreeRtos::delay_ms(2000);
                }
            });
        }
        Err(e) => {
            error!("Failed to init sensor: {}", e);
            // 如果失败，让LED继续闪着
            loop {
                FreeRtos::delay_ms(1000);
            }
        }
    }

    Ok(())
}
