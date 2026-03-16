use esp_idf_sys as sys;
use log::info;

/// BLE广播模块 - 使用ESP-IDF 5.x Extended Advertising API

/// 初始化BLE并开始广播
pub fn start_ble_advertising(co2_value: u16) {
    unsafe {
        info!("BLE: Starting...");

        // 先尝试停止任何已存在的 BLE 控制器
        let _ = sys::esp_bt_controller_disable();
        let _ = sys::esp_bt_controller_deinit();

        // 等待一下让系统清理
        for _ in 0..1000 {
            std::thread::yield_now();
        }

        // 获取默认配置并设置
        let mut bt_config = sys::esp_bt_controller_config_t::default();

        info!("BLE: Init controller...");
        let ret = sys::esp_bt_controller_init(&mut bt_config);
        if ret != 0 {
            info!("BLE: BT controller init failed: {}", ret);
            return;
        }
        info!("BLE: Controller init OK");

        // 启用BLE模式
        let ret = sys::esp_bt_controller_enable(sys::esp_bt_mode_t_ESP_BT_MODE_BLE);
        if ret != 0 {
            info!("BLE: BT enable failed: {}", ret);
            return;
        }
        info!("BLE: Controller enable OK");

        // 初始化bluedroid
        let ret = sys::esp_bluedroid_init();
        if ret != 0 {
            info!("BLE: Bluedroid init failed: {}", ret);
            return;
        }
        info!("BLE: Bluedroid init OK");

        // 启用bluedroid
        let ret = sys::esp_bluedroid_enable();
        if ret != 0 {
            info!("BLE: Bluedroid enable failed: {}", ret);
            return;
        }
        info!("BLE: Bluedroid enable OK");

        // 配置扩展广播参数
        let mut ext_adv_params = sys::esp_ble_gap_ext_adv_params_t::default();
        ext_adv_params.type_ = 1;
        ext_adv_params.interval_min = 0x100;
        ext_adv_params.interval_max = 0x100;
        ext_adv_params.channel_map = 7;
        ext_adv_params.own_addr_type = sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC;
        ext_adv_params.peer_addr_type = sys::esp_ble_addr_type_t_BLE_ADDR_TYPE_PUBLIC;
        ext_adv_params.filter_policy = 0;
        ext_adv_params.tx_power = 0x7f;
        ext_adv_params.primary_phy = 1;
        ext_adv_params.secondary_phy = 1;
        ext_adv_params.sid = 0;
        info!("BLE: Params configured");

        // 设置扩展广播参数
        let ret = sys::esp_ble_gap_ext_adv_set_params(0, &ext_adv_params);
        if ret != 0 {
            info!("BLE: Set ext adv params failed: {}", ret);
            return;
        }
        info!("BLE: Set params OK");

        // 配置广播数据
        update_adv_data(co2_value);

        // 开始扩展广播
        let mut ext_adv = sys::esp_ble_gap_ext_adv_t::default();
        ext_adv.instance = 0;
        ext_adv.duration = 0;
        ext_adv.max_events = 0;

        let ret = sys::esp_ble_gap_ext_adv_start(1, &ext_adv);
        if ret != 0 {
            info!("BLE: Start ext adv failed: {}", ret);
            return;
        }

        info!("BLE: Advertising started!");
    }
}

/// 更新广播数据
fn update_adv_data(co2_value: u16) {
    unsafe {
        let mut raw_data = [0u8; 31];
        let mut pos = 0;

        // 设备名称 "CO2"
        let name = b"CO2";
        raw_data[pos] = name.len() as u8 + 1;
        raw_data[pos + 1] = 0x09;
        raw_data[pos + 2..pos + 2 + name.len()].copy_from_slice(name);
        pos += 2 + name.len();

        // Manufacturer Data
        raw_data[pos] = 4;
        raw_data[pos + 1] = 0xFF;
        raw_data[pos + 2] = 0x12;
        raw_data[pos + 3] = 0x34;
        let co2_bytes = co2_value.to_le_bytes();
        raw_data[pos + 4] = co2_bytes[0];
        raw_data[pos + 5] = co2_bytes[1];
        pos += 6;

        let ret = sys::esp_ble_gap_config_ext_adv_data_raw(0, pos as u16, raw_data.as_mut_ptr());
        if ret != 0 {
            info!("BLE: Config adv data failed: {}", ret);
        } else {
            info!("BLE: Adv data OK");
        }
    }
}

/// 更新CO2值
pub fn update_co2_value(co2_value: u16) {
    update_adv_data(co2_value);
    info!("BLE CO2: {} ppm", co2_value);
}
