import tkinter as tk
from tkinter import ttk
import serial
import serial.tools.list_ports
import threading
import time

class CO2MonitorGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Premium CO2 Monitor")
        self.root.geometry("400x500")
        self.root.configure(bg="#1e1e1e")
        
        self.current_co2 = 0
        self.is_connected = False
        
        # --- UI Components ---
        self.create_widgets()
        
        # --- Serial Thread ---
        self.running = True
        self.serial_thread = threading.Thread(target=self.serial_loop, daemon=True)
        self.serial_thread.start()
        
    def create_widgets(self):
        # Header
        header = tk.Label(self.root, text="室内空气质量监测", font=("Microsoft YaHei", 18, "bold"), 
                         fg="#ffffff", bg="#1e1e1e", pady=20)
        header.pack()
        
        # Value Display Box
        self.value_frame = tk.Frame(self.root, bg="#2d2d2d", padx=30, pady=30, 
                                   highlightbackground="#444", highlightthickness=1)
        self.value_frame.pack(pady=20, expand=True)
        
        self.label_value = tk.Label(self.value_frame, text="000", font=("DIN Condensed", 72, "bold"), 
                                   fg="#4caf50", bg="#2d2d2d")
        self.label_value.pack()
        
        self.label_unit = tk.Label(self.value_frame, text="ppm", font=("Microsoft YaHei", 14), 
                                  fg="#888888", bg="#2d2d2d")
        self.label_unit.pack()
        
        # Status Label
        self.label_status = tk.Label(self.root, text="正在搜索设备...", font=("Microsoft YaHei", 10), 
                                    fg="#ff9800", bg="#1e1e1e", pady=10)
        self.label_status.pack()
        
        # Footer
        footer = tk.Label(self.root, text="Data via Serial Port", font=("Microsoft YaHei", 8), 
                         fg="#555555", bg="#1e1e1e")
        footer.pack(side="bottom", pady=10)

    def update_ui(self, val):
        self.label_value.config(text=f"{val:03d}")
        
        # Dynamic Color Logic
        if val < 800:
            color = "#4caf50" # Green
            status_txt = "空气清新"
        elif val < 1200:
            color = "#ff9800" # Orange
            status_txt = "注意通风"
        else:
            color = "#f44336" # Red
            status_txt = "迅速开窗！浓度超标"
            
        self.label_value.config(fg=color)
        self.label_status.config(text=status_txt, fg=color)
        
        # Subtle flash effect on update
        orig_bg = self.value_frame.cget("bg")
        self.value_frame.config(bg="#3d3d3d")
        self.root.after(100, lambda: self.value_frame.config(bg=orig_bg))

    def find_port(self):
        ports = serial.tools.list_ports.comports()
        for port in ports:
            hwid_desc = f"{port.hwid} {port.description}".lower()
            if any(x in hwid_desc for x in ["cp210", "silicon labs", "ch340", "usb serial"]):
                return port.device
        return None

    def serial_loop(self):
        while self.running:
            port = self.find_port()
            if not port:
                self.label_status.config(text="未检测到设备，请插入 USB", fg="#888")
                time.sleep(2)
                continue
            
            try:
                with serial.Serial(port, 115200, timeout=1) as ser:
                    self.is_connected = True
                    # Clean buffer
                    ser.reset_input_buffer()
                    
                    while self.running:
                        line = ser.readline().decode('utf-8', errors='ignore').strip()
                        if line.startswith("DATA:CO2:"):
                            try:
                                val = int(line.split(":")[2])
                                self.root.after(0, self.update_ui, val)
                            except (ValueError, IndexError):
                                pass
                        elif "ERROR" in line:
                            self.root.after(0, lambda: self.label_status.config(text="传感器故障！", fg="#f00"))
            except Exception as e:
                self.is_connected = False
                time.sleep(2)

if __name__ == "__main__":
    root = tk.Tk()
    app = CO2MonitorGUI(root)
    root.mainloop()
