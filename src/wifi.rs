use std::sync::atomic::{AtomicU16, Ordering};
use esp_idf_hal::io::Write;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi};
use log::info;

static CO2_VALUE: AtomicU16 = AtomicU16::new(0);

pub fn update_co2(value: u16) {
    CO2_VALUE.store(value, Ordering::SeqCst);
}

pub fn get_co2() -> u16 {
    CO2_VALUE.load(Ordering::SeqCst)
}

/// 获取 HTML 页面
pub fn get_html() -> String {
    let co2 = get_co2();
    let color = if co2 < 800 {
        "#4caf50"
    } else if co2 < 1200 {
        "#ff9800"
    } else {
        "#f44336"
    };
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>CO2 Monitor</title>
    <style>
        body {{ font-family: Arial, sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #f5f5f5; }}
        .card {{ background: white; padding: 40px; border-radius: 16px; box-shadow: 0 4px 12px rgba(0,0,0,0.1); text-align: center; }}
        h1 {{ color: #333; margin-bottom: 10px; }}
        .co2 {{ font-size: 72px; font-weight: bold; color: {}; margin: 20px 0; }}
        .unit {{ font-size: 24px; color: #666; }}
        .status {{ margin-top: 20px; font-size: 14px; color: #999; }}
    </style>
    <meta http-equiv="refresh" content="2">
</head>
<body>
    <div class="card">
        <h1>CO2 Monitor</h1>
        <div class="co2">{}</div>
        <div class="unit">ppm</div>
        <div class="status">刷新率: 2秒 | ESP32-S3</div>
    </div>
</body>
</html>"#,
        color, co2
    )
}

/// 启动 WiFi AP 模式，返回 (BlockingWifi, EspHttpServer) 以防止被 drop
pub fn start_ap(
    modem: Modem,
    ssid: &str,
    password: &str,
) -> anyhow::Result<(BlockingWifi<EspWifi<'static>>, EspHttpServer<'static>)> {
    info!("WiFi: Starting AP mode...");

    // SAFETY: Peripherals::take() 保证全局唯一，modem 与整个程序同生命周期
    let modem_static: Modem<'static> = unsafe { core::mem::transmute(modem) };

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let esp_wifi = EspWifi::new(modem_static, sys_loop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

    info!("WiFi: Creating AP configuration...");
    wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
        ssid: ssid.try_into().map_err(|_| anyhow::anyhow!("SSID too long"))?,
        password: password.try_into().map_err(|_| anyhow::anyhow!("Password too long"))?,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    info!("WiFi: Starting AP...");
    wifi.start()?;

    info!("WiFi AP started! SSID: {}, IP: 192.168.4.1", ssid);

    // 启动 HTTP 服务器
    let server = start_http_server()?;

    Ok((wifi, server))
}

/// 启动 HTTP 服务器
fn start_http_server() -> anyhow::Result<EspHttpServer<'static>> {
    info!("HTTP: Starting server on port 80...");

    let mut server = EspHttpServer::new(&HttpConfig::default())?;

    server.fn_handler("/", Method::Get, |req| {
        let html = get_html();
        req.into_ok_response()?.write_all(html.as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    info!("HTTP: Server ready! Visit http://192.168.4.1");
    Ok(server)
}
