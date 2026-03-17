#include <stdio.h>
#include <stdint.h>

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"
#include "esp_system.h"
#include "esp_task_wdt.h"
#include "mhz19.h"
#include "led_strip_app.h"

static const char *TAG = "MAIN";

// MH-Z19 接线：S3 GPIO1 TX -> 传感器 RX, GPIO2 RX -> 传感器 TX
#define MHZ19_TX_PIN 1
#define MHZ19_RX_PIN 2

// 看门狗超时时间 (毫秒)
#define WDT_TIMEOUT_MS 5000

void co2_monitor_task(void *pvParameters) {
    uint16_t co2 = 0;
    
    // 将当前任务加入看门狗监控
    esp_task_wdt_add(NULL);
    
    while (1) {
        // 喂狗，表示任务还在运行
        esp_task_wdt_reset();
        
        if (mhz19_read_co2(&co2) == ESP_OK) {
            // 串口协议格式，方便 GUI 解析
            printf("DATA:CO2:%u\n", co2);
            ESP_LOGI(TAG, "CO2 Concentration: %u ppm", co2);
        } else {
            printf("DATA:ERROR:SENSOR_FAIL\n");
            ESP_LOGE(TAG, "Failed to read CO2 sensor");
        }
        
        vTaskDelay(pdMS_TO_TICKS(1000));
    }
}

void app_main(void) {
    // 1. 启动延迟：给电源和传感器 2 秒稳定时间，防止上电时的瞬间波动导致初始化失败
    ESP_LOGI(TAG, "系统启动中，正在等待电源稳定 (2s)...");
    vTaskDelay(pdMS_TO_TICKS(2000));

    // 2. 打印重启原因，用于故障排查
    esp_reset_reason_t reason = esp_reset_reason();
    ESP_LOGI(TAG, "上次重启原因: %d", reason);
    if (reason == ESP_RST_TASK_WDT) {
        ESP_LOGW(TAG, "检测到任务超时，已通过看门狗自动复位！");
    }

    // 3. 配置并启动看门狗
    esp_task_wdt_config_t twdt_config = {
        .timeout_ms = WDT_TIMEOUT_MS,
        .idle_core_mask = 0, // 仅监控订阅的任务，不强制监控空闲任务
        .trigger_panic = true,
    };
    esp_task_wdt_init(&twdt_config);

    // 4. 初始化硬件
    ESP_ERROR_CHECK(mhz19_init(MHZ19_TX_PIN, MHZ19_RX_PIN));
    led_strip_app_start();

    // 5. 启动 CO2 监测任务
    xTaskCreate(co2_monitor_task, "co2_monitor_task", 4096, NULL, 5, NULL);
    
    ESP_LOGI(TAG, "系统初始化完成，看门狗已启用");
}
