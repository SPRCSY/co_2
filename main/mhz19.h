#ifndef MHZ19_H
#define MHZ19_H

#include <stdint.h>
#include "driver/uart.h"

#define MHZ19_UART_PORT UART_NUM_2
#define MHZ19_BAUD_RATE 9600

esp_err_t mhz19_init(int tx_pin, int rx_pin);
esp_err_t mhz19_read_co2(uint16_t *co2_ppm);

#endif // MHZ19_H
