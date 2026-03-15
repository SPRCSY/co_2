use esp_idf_hal::{
    gpio::{InputPin, OutputPin},
    uart::{AsyncUartDriver, Uart, UartConfig, UartDriver},
    units::Hertz,
};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Uart(esp_idf_hal::sys::EspError),
    InvalidHeader,
    ChecksumError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Uart(e) => write!(f, "UART error: {}", e),
            Error::InvalidHeader => write!(f, "Invalid header received"),
            Error::ChecksumError => write!(f, "Checksum validation failed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<esp_idf_hal::sys::EspError> for Error {
    fn from(err: esp_idf_hal::sys::EspError) -> Self {
        Error::Uart(err)
    }
}

pub struct MHZ19<'a> {
    uart: AsyncUartDriver<'a, UartDriver<'a>>,
}

impl<'a> MHZ19<'a> {
    /// 构造函数，初始化 UART 波特率为 9600 并开启异步驱动
    pub fn new(
        uart_port: impl Uart + 'a,
        tx_pin: impl OutputPin + 'a,
        rx_pin: impl InputPin + 'a,
    ) -> Result<Self, Error> {
        let config = UartConfig::new().baudrate(Hertz(9600));

        let async_uart = AsyncUartDriver::new(
            uart_port,
            tx_pin,
            rx_pin,
            Option::<esp_idf_hal::gpio::AnyIOPin>::None,
            Option::<esp_idf_hal::gpio::AnyIOPin>::None,
            &config,
        )?;

        Ok(MHZ19 { uart: async_uart })
    }

    /// 异步读取 CO2 浓度
    pub async fn read_co2(&mut self) -> Result<u16, Error> {
        let cmd: [u8; 9] = [0xFF, 0x01, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79];

        // 发送 9 字节读取命令
        self.uart.write(&cmd).await?;

        // 接收 9 字节响应
        let mut buf = [0u8; 9];
        let mut len = 0;

        // 持续读取直到满足 9 个字节
        let mut read_attempts = 0;
        while len < 9 {
            let read_bytes = self.uart.read(&mut buf[len..]).await?;
            if read_bytes == 0 {
                // 如果连续没有读到数据，休眠并增加尝试读计数器来防止死锁
                esp_idf_hal::delay::FreeRtos::delay_ms(100);
                read_attempts += 1;
                if read_attempts > 20 { // 2秒超时
                    // 如果一直没有读到返回，直接报错
                    return Err(Error::InvalidHeader); // 可以临时用这个返回，或者你可以之后重加上 Timeout 变体
                }
                continue;
            }
            len += read_bytes;
            read_attempts = 0; // 重置超时计数
        }

        // 验证报头 (字节 0 必须为 0xFF, 字节 1 必须为 0x86)
        if buf[0] != 0xFF || buf[1] != 0x86 {
            return Err(Error::InvalidHeader);
        }

        // CRC 和校验 (字节 1 到字节 8 的总和，其低 8 位必须为 0)
        let sum: u8 = buf[1..=8].iter().fold(0, |acc, &b| acc.wrapping_add(b));
        if sum != 0 {
            return Err(Error::ChecksumError);
        }

        // 计算 CO2 浓度：第 2 字节为高八位，第 3 字节为低八位 (0-indexed)
        let high = buf[2] as u16;
        let low = buf[3] as u16;
        let co2_concentration = (high << 8) | low;

        Ok(co2_concentration)
    }
}
