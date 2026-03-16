"""
CO2 Monitor BLE 客户端
使用 bleak 库扫描并读取 ESP32 广播的 CO2 数据

安装依赖:
    pip install bleak

运行:
    python ble_client.py --continuous
"""

import asyncio
from bleak import BleakScanner
import struct
import sys

# ESP32 广播的设备名
DEVICE_NAME = "CO2-MON"

def parse_co2_from_adv(data: bytes) -> int | None:
    """从广播数据中解析 CO2 值"""
    i = 0
    while i < len(data):
        length = data[i]
        if length == 0 or i + 1 >= len(data):
            break
        ad_type = data[i + 1]
        if ad_type == 0xFF:  # Manufacturer Data
            # 格式: [长度, 0xFF, vendor_low, vendor_high, co2_low, co2_high]
            if length >= 4 and i + length < len(data):
                co2_bytes = data[i+4:i+6]
                if len(co2_bytes) == 2:
                    co2 = struct.unpack('<H', co2_bytes)[0]
                    return co2
        i += length + 1
    return None

async def continuous_scan():
    """持续扫描模式"""
    print(f"持续扫描 {DEVICE_NAME}...")
    print("按 Ctrl+C 退出\n")

    def callback(device, advertising_data):
        if device.name and DEVICE_NAME in device.name:
            # 解析原始广播数据
            raw_data = advertising_data.bytes
            co2 = parse_co2_from_adv(raw_data)
            if co2:
                print(f"[{device.address}] CO2: {co2} ppm")

    scanner = BleakScanner(callback)

    try:
        await scanner.start()
        print("扫描中...\n")
        while True:
            await asyncio.sleep(0.5)
    except KeyboardInterrupt:
        print("\n退出")
    finally:
        await scanner.stop()

def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--continuous", action="store_true", help="持续扫描模式")
    args = parser.parse_args()

    if args.continuous:
        asyncio.run(continuous_scan())
    else:
        print("单次扫描模式...")
        async def scan_once():
            devices = await BleakScanner.discover(timeout=5)
            found = False
            for d in devices:
                if d.name and DEVICE_NAME in d.name:
                    print(f"找到: {d.name} ({d.address})")
                    found = True
            if not found:
                print("未找到设备")
        asyncio.run(scan_once())

if __name__ == "__main__":
    main()
