idf_component_register(SRCS "main.c" "mhz19.c" "wifi_app.c" "led_strip_app.c"
                    INCLUDE_DIRS "."
                    PRIV_REQUIRES esp_http_server nvs_flash esp_wifi led_strip esp_driver_uart mdns)



