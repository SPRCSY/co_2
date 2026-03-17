#ifndef WIFI_APP_H
#define WIFI_APP_H

#include <stdint.h>
#include "esp_err.h"

void wifi_app_start(void);
void wifi_app_update_co2(uint16_t co2);

#endif // WIFI_APP_H
