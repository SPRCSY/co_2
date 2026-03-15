import serial
import serial.tools.list_ports
import time
import datetime
import csv
import sys
import os

def find_esp32_port():
    """遍历串口，寻找 ESP32，匹配 CP2102、Silicon Labs 或 COM9"""
    while True:
        ports = serial.tools.list_ports.comports()
        for port in ports:
            hwid_desc = f"{port.hwid} {port.description}".lower()
            # 常见 ESP32 DevKit 转换芯片标志
            if "cp210" in hwid_desc or "silicon labs" in hwid_desc:
                return port.device
            # 兼容开发过程中的固定口 COM9
            if "com9" == port.device.lower():
                return port.device
                
        print("未检测到 ESP32 连接 (查找 CP2102/Silicon Labs，或 COM9)。5 秒后重试...")
        time.sleep(5)

def main():
    baudrate = 115200
    date_str = datetime.datetime.now().strftime("%Y%m%d")
    csv_filename = f"co2_{date_str}.csv"
    
    write_header = not os.path.exists(csv_filename)

    print(f"日志将保存到: {csv_filename}")
    
    # 初始化文件并写入表头
    try:
        with open(csv_filename, "a", newline="", encoding="utf-8") as f:
            writer = csv.writer(f)
            if write_header:
                writer.writerow(["Timestamp", "CO2 Value(ppm)"])
    except Exception as e:
        print(f"无法打开日志文件: {e}")
        return

    while True:
        port_name = find_esp32_port()
        print(f"尝试连接到 {port_name}...")
        
        try:
            with serial.Serial(port_name, baudrate, timeout=1) as ser:
                print(f"已成功连接到 {port_name}，波特率 {baudrate}")
                while True:
                    if ser.in_waiting > 0:
                        # 读取一行并去除空白和无效字节
                        line = ser.readline().decode('utf-8', errors='ignore').strip()
                        
                        # 解析数据行，支持纯"CO2: 450"或者附带日志前缀如"I (4532) main: CO2: 450"
                        if "CO2:" in line:
                            try:
                                co2_val_str = line.split("CO2:")[1].strip()
                                co2_val = int(co2_val_str)
                                timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                                
                                # 打印到控制台
                                print(f"[{timestamp}] CO2 浓度: {co2_val} ppm")
                                
                                # 将数据持续追加保存至 CSV (每次打开保证断电保护)
                                with open(csv_filename, "a", newline="", encoding="utf-8") as f:
                                    writer = csv.writer(f)
                                    writer.writerow([timestamp, co2_val])
                            except ValueError:
                                pass
                        elif "ERR" in line:
                            print(f"⚠️ {line}")
                            
        except serial.SerialException as e:
            print(f"串口异常或 ESP32 断开连接: {e}")
            time.sleep(2)
        except KeyboardInterrupt:
            print("\n程序手动退出。")
            sys.exit(0)

if __name__ == "__main__":
    main()
