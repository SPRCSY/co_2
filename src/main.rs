use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::delay::FreeRtos;
use log::{info, error};


mod mhz19;
use mhz19::MHZ19;

mod ble;
mod wifi;
mod led_status;


fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting CO2 Monitor...");

    let peripherals = Peripherals::take().unwrap();

    // RGB LED 状态指示线程（绿→熄→蓝→熄，各1s循环）
    // GPIO48 接 WS2812 数据线，使用 RMT channel 0
    led_status::spawn_led_status_thread(peripherals.pins.gpio48, 0);


    // 初始化 UART2 (GPIO1 TX, GPIO2 RX)
    info!("Initializing UART2 on GPIO1/GPIO2...");
    let uart2 = peripherals.uart2;
    let tx = peripherals.pins.gpio1;
    let rx = peripherals.pins.gpio2;

    // WiFi AP 模式，持有返回值以防止 drop
    let _wifi_handles = wifi::start_ap(peripherals.modem, "CO2-Monitor", "12345678")?;

    match MHZ19::new(uart2, tx, rx) {
        Ok(mut sensor) => {
            info!("Sensor initialized");

            esp_idf_svc::hal::task::block_on(async {
                loop {
                    match sensor.read_co2().await {
                        Ok(co2) => {
                            info!("CO2: {}", co2);
                            wifi::update_co2(co2);
                        }
                        Err(e) => {
                            error!("ERR: {}", e);
                        }
                    }
                    FreeRtos::delay_ms(2000);
                }
            });
        }
        Err(e) => {
            error!("Failed to init sensor: {}", e);
            loop {
                FreeRtos::delay_ms(1000);
            }
        }
    }

    Ok(())
}
