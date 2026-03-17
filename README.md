# CO2 Monitor (Multi-Mode / High Availability)

基于 ESP32-S3-DevKitC-V1.0 和 MH-Z19 传感器的二氧化碳监测方案。本项目经历了从 Rust 到 C 语言的优化演进，目前支持多种监控模式。

## 🌟 核心功能 (C 语言版 - 推荐)
- **高性能串口监测**：采用原生 ESP-IDF C 开发，通过 `co2_gui.py` 桌面程序实时展示数据（适配仅 5G WiFi 环境）。
- **硬件级自愈**：内置 2s 启动预热延迟与 5s 任务看门狗，彻底解决因拔插导致的死机问题。
- **高颜值 GUI**：深色模式监控看板，支持浓度预警动态变色。
- **SmartConfig 备份**：完整保留 STA 配网与 HTTP 监控接口代码，随时可恢复无线监测。

## 🛠 硬件规格
- **芯片**: ESP32-S3-DevKitC-V1.0 (8MB PSRAM)
- **传感器**: MH-Z19D (CO2)
- **接线**: 
  - GPIO1 (TX) -> 传感器 RX
  - GPIO2 (RX) -> 传感器 TX
  - GPIO48 -> 板载 WS2812 RGB LED

---

## 🚀 快速开始

### 1. 硬件端 (C 语言版)
```bash
# 构建并烧录
idf.py set-target esp32s3
idf.py build
idf.py flash
```

### 2. 软件端 (Python GUI)
```bash
pip install pyserial
python co2_gui.py
```

---

## 🏛 历史版本与兼容性

### Rust 版 (已存档)
本项目最初采用 Rust 开发。相关源码保留在 `src/`, `Cargo.toml` 等文件中。
```bash
cargo build
cargo espflash flash --monitor
```

### BLE 扩展广播 (已开发)
支持通过低功耗蓝牙 (BLE Extended Advertising) 广播浓度数据。虽然当前版本主打串口，但相关驱动逻辑已保留。
- **Manufacturer Data**: 包含实时的 ppm 数值。
- **客户端**: 见 `ble_client.py`。

## 📂 项目结构
- `main/`: ESP-IDF C 源码及其驱动。
- `src/`: 原始 Rust 版本的源码。
- `co2_gui.py`: 当前推荐的桌面 GUI 监控程序。
- `co2_monitor.py`: 基础命令行串口日志工具。
- `README.md`: 项目全景说明。

---
*本项目保留了所有开发阶段的可用代码。默认运行最稳定的串口 GUI 模式。*
