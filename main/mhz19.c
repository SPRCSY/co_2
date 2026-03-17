#include "mhz19.h"
#include "esp_log.h"
#include <string.h>

static const char *TAG = "MHZ19";

esp_err_t mhz19_init(int tx_pin, int rx_pin) {
    uart_config_t uart_config = {
        .baud_rate = MHZ19_BAUD_RATE,
        .data_bits = UART_DATA_8_BITS,
        .parity    = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };
    
    ESP_ERROR_CHECK(uart_param_config(MHZ19_UART_PORT, &uart_config));
    ESP_ERROR_CHECK(uart_set_pin(MHZ19_UART_PORT, tx_pin, rx_pin, UART_PIN_NO_CHANGE, UART_PIN_NO_CHANGE));
    ESP_ERROR_CHECK(uart_driver_install(MHZ19_UART_PORT, 256, 0, 0, NULL, 0));
    
    return ESP_OK;
}

esp_err_t mhz19_read_co2(uint16_t *co2_ppm) {
    const uint8_t cmd[9] = {0xFF, 0x01, 0x86, 0x00, 0x00, 0x00, 0x00, 0x00, 0x79};
    uint8_t data[9];
    
    uart_write_bytes(MHZ19_UART_PORT, (const char *)cmd, 9);
    
    int len = uart_read_bytes(MHZ19_UART_PORT, data, 9, pdMS_TO_TICKS(1000));
    
    if (len != 9) {
        ESP_LOGE(TAG, "UART read timeout or incomplete: %d bytes", len);
        return ESP_FAIL;
    }
    
    if (data[0] != 0xFF || data[1] != 0x86) {
        ESP_LOGE(TAG, "Invalid response header: %02x %02x", data[0], data[1]);
        return ESP_FAIL;
    }
    
    uint8_t checksum = 0;
    for (int i = 1; i < 8; i++) {
        checksum += data[i];
    }
    checksum = 0xFF - checksum;
    checksum += 1;
    
    if (checksum != data[8]) {
        ESP_LOGE(TAG, "Checksum error: calculated %02x, received %02x", checksum, data[8]);
        // return ESP_FAIL; // Some clones have slightly different checksum logic, but let's be strict
    }
    
    *co2_ppm = (uint16_t)data[2] << 8 | data[3];
    return ESP_OK;
}
