use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::rmt::{Pulse, PinState, TxRmtDriver, config::TransmitConfig};
use smart_leds::RGB8;

/// 启动 RGB LED 状态指示线程
/// 
/// 闪烁规律：绿1s → 熄1s → 蓝1s → 熄1s，循环
/// 
/// 注意：为了彻底解决外部依赖库导致的 esp-idf-sys 版本冲突（links 冲突），
/// 我们直接在项目内部实现一个轻量级的 RMT 驱动。
pub fn spawn_led_status_thread<P>(pin: P, channel: u8)
where
    P: OutputPin + Peripheral<P = P> + Send + 'static,
{
    std::thread::Builder::new()
        .name("led_status".to_string())
        .stack_size(4096)
        .spawn(move || {
            // RMT 配置，针对 WS2812B 调优
            let config = TransmitConfig::new().clock_divider(1); // 80MHz
            let mut driver = match TxRmtDriver::new(channel, pin, &config) {
                Ok(d) => d,
                Err(e) => {
                    log::error!("LED RMT init failed: {:?}", e);
                    return;
                }
            };

            // WS2812 脉冲宽度定义（单位：ticks @ 80MHz = 12.5ns/tick）
            let t0h = 32; // 400ns
            let t0l = 68; // 850ns
            let t1h = 64; // 800ns
            let t1l = 36; // 450ns

            let off   = RGB8 { r: 0, g: 0, b: 0 };
            let green = RGB8 { r: 0, g: 32, b: 0 };
            let blue  = RGB8 { r: 0, g: 0, b: 32 };

            let mut send_color = |color: RGB8| {
                let mut pulses = Vec::with_capacity(24);
                // WS2812: G, R, B 顺序
                let bits = ((color.g as u32) << 16) | ((color.r as u32) << 8) | (color.b as u32);
                
                for i in (0..24).rev() {
                    let bit = (bits >> i) & 1;
                    if bit == 1 {
                        pulses.push(Pulse::new(PinState::High, t1h.into(), PinState::Low, t1l.into()));
                    } else {
                        pulses.push(Pulse::new(PinState::High, t0h.into(), PinState::Low, t0l.into()));
                    }
                }
                let _ = driver.start_blocking(&pulses);
            };

            loop {
                send_color(green);
                std::thread::sleep(std::time::Duration::from_secs(1));

                send_color(off);
                std::thread::sleep(std::time::Duration::from_secs(1));

                send_color(blue);
                std::thread::sleep(std::time::Duration::from_secs(1));

                send_color(off);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
        .expect("Failed to spawn LED status thread");
}
