# CO2 Monitor

基于 ESP32-S3 和 MH-Z19D 传感器的二氧化碳监测仪。

## 硬件

- **开发板**: ESP32-S3-DevKitC-V1.0
- **传感器**: MH-Z19D (CO2)
- **接口**: UART2 (GPIO1 TX, GPIO2 RX)
- **LED**: GPIO48 (板载 RGB LED)

## 功能

- [x] CO2 浓度读取 (5秒刷新)
- [x] RGB LED 状态指示 (GPIO48)
- [x] 串口输出 (115200波特率)
- [ ] BLE 无线传输 (开发中)

## 快速开始

### 构建

```bash
cargo build
```

### 烧录

```bash
cargo espflash flash --monitor
```

### Python 上位机 (串口)

```bash
python co2_monitor.py
```

### BLE 客户端 (待完善)

```bash
# 安装依赖
pip install bleak

# 运行 BLE 扫描
python ble_client.py --continuous
```

## 引脚分配

| 功能 | GPIO |
|------|------|
| CO2 TX | GPIO1 |
| CO2 RX | GPIO2 |
| LED | GPIO48 |

## BLE 服务 (计划中)

将添加 BLE GATT 服务，通过手机读取 CO2 数据。
