#include "wifi_app.h"
#include <string.h>
#include <sys/param.h>
#include "esp_wifi.h"
#include "esp_event.h"
#include "esp_log.h"
#include "nvs_flash.h"
#include "esp_http_server.h"
#include "freertos/FreeRTOS.h"
#include "freertos/event_groups.h"
#include "freertos/task.h"
#include "lwip/sockets.h"
#include "mdns.h"
#include "esp_smartconfig.h"

static const char *TAG = "WIFI_APP";
static uint16_t current_co2 = 0;
static httpd_handle_t server = NULL;

#define WIFI_CONNECTED_BIT BIT0
#define ESPTOUCH_DONE_BIT  BIT1
static EventGroupHandle_t s_wifi_event_group;
static int s_retry_num = 0;

static char wifi_status_msg[64] = "未配置 WiFi";
static char current_sta_ssid[32] = "";
static char current_sta_ip[16] = "0.0.0.0";
static bool smartconfig_done = false;

void wifi_app_update_co2(uint16_t co2) {
    current_co2 = co2;
}

// --- NVS Helpers ---

static esp_err_t save_wifi_config(const char *ssid, const char *password) {
    nvs_handle_t my_handle;
    esp_err_t err = nvs_open("storage", NVS_READWRITE, &my_handle);
    if (err != ESP_OK) return err;
    err = nvs_set_str(my_handle, "ssid", ssid);
    if (err == ESP_OK) err = nvs_set_str(my_handle, "password", password);
    if (err == ESP_OK) err = nvs_commit(my_handle);
    nvs_close(my_handle);
    return err;
}

static esp_err_t load_wifi_config(char *ssid, size_t ssid_len, char *password, size_t pass_len) {
    nvs_handle_t my_handle;
    esp_err_t err = nvs_open("storage", NVS_READONLY, &my_handle);
    if (err != ESP_OK) return err;
    err = nvs_get_str(my_handle, "ssid", ssid, &ssid_len);
    if (err == ESP_OK) err = nvs_get_str(my_handle, "password", password, &pass_len);
    nvs_close(my_handle);
    return err;
}

// --- mDNS & HTTP ---

static void initialise_mdns(void) {
    mdns_init();
    mdns_hostname_set("co2");
    mdns_instance_name_set("ESP32-S3 CO2 Monitor");
    mdns_service_add(NULL, "_http", "_tcp", 80, NULL, 0);
}

static esp_err_t api_co2_get_handler(httpd_req_t *req) {
    char json[64];
    snprintf(json, sizeof(json), "{\"co2\": %u}", current_co2);
    httpd_resp_set_type(req, "application/json");
    httpd_resp_send(req, json, HTTPD_RESP_USE_STRLEN);
    return ESP_OK;
}

static esp_err_t index_get_handler(httpd_req_t *req) {

    char color[8];
    if (current_co2 < 800) strcpy(color, "#4caf50");
    else if (current_co2 < 1200) strcpy(color, "#ff9800");
    else strcpy(color, "#f44336");

    char html[1600];
    snprintf(html, sizeof(html),
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
        "<title>CO2 Monitor</title><style>body { font-family: sans-serif; display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; margin: 0; background: #f0f2f5; }"
        ".card { background: white; padding: 2rem; border-radius: 1rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); text-align: center; width: 80%%; max-width: 400px; }"
        ".co2 { font-size: 5rem; font-weight: bold; color: %s; margin: 1rem 0; }"
        ".unit { font-size: 1.5rem; color: #65676b; }"
        ".status { margin-top: 1.5rem; padding: 0.8rem; border-radius: 0.5rem; background: #e9ecef; font-size: 0.85rem; color: #495057; text-align: left; }"
        ".ip { font-family: monospace; color: #007bff; font-weight: bold; }</style>"
        "<meta http-equiv=\"refresh\" content=\"3\"></head><body><div class=\"card\"><h1>CO2 Monitor</h1><div class=\"co2\">%u</div><div class=\"unit\">ppm</div>"
        "<div class=\"status\">"
        "<div>WiFi 状态: <b>%s</b></div>"
        "<div>内网 IP: <span class=\"ip\">%s</span></div>"
        "<div>内网域名: <span class=\"ip\">co2.local</span></div>"
        "</div>"
        "<p style=\"font-size:0.7rem;color:#666;margin-top:1rem;\">可通过微信或 Esptouch App 进行一键配网</p>"
        "</div></body></html>",
        color, current_co2, wifi_status_msg, current_sta_ip);

    httpd_resp_send(req, html, HTTPD_RESP_USE_STRLEN);
    return ESP_OK;
}

// --- SmartConfig Task ---

static void smartconfig_task(void * parm) {
    EventBits_t uxBits;
    ESP_ERROR_CHECK( esp_smartconfig_set_type(SC_TYPE_ESPTOUCH) );
    smartconfig_start_config_t cfg = SMARTCONFIG_START_CONFIG_DEFAULT();
    ESP_ERROR_CHECK( esp_smartconfig_start(&cfg) );
    ESP_LOGI(TAG, "SmartConfig started...");
    
    while (1) {
        uxBits = xEventGroupWaitBits(s_wifi_event_group, WIFI_CONNECTED_BIT | ESPTOUCH_DONE_BIT, true, false, portMAX_DELAY);
        if (uxBits & WIFI_CONNECTED_BIT) {
            ESP_LOGI(TAG, "WiFi Connected through SmartConfig");
        }
        if (uxBits & ESPTOUCH_DONE_BIT) {
            ESP_LOGI(TAG, "SmartConfig complete, restarting...");
            esp_smartconfig_stop();
            vTaskDelay(pdMS_TO_TICKS(1000));
            esp_restart();
        }
    }
}

// --- Event Handler ---

static void event_handler(void* arg, esp_event_base_t event_base, int32_t event_id, void* event_data) {
    if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {
        esp_wifi_connect();
        strcpy(wifi_status_msg, "正在尝试连接已存 WiFi...");
    } else if (event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {
        wifi_event_sta_disconnected_t* event = (wifi_event_sta_disconnected_t*) event_data;
        ESP_LOGE(TAG, "STA Disconnected, reason: %d", event->reason);
        strcpy(current_sta_ip, "0.0.0.0");
        
        if (s_retry_num < 5) {
            esp_wifi_connect();
            s_retry_num++;
            snprintf(wifi_status_msg, sizeof(wifi_status_msg), "连接失败，正在重试 %d/5", s_retry_num);
        } else if (!smartconfig_done) {
            strcpy(wifi_status_msg, "由于连接失败，已启动 SmartConfig 等待配网...");
            xTaskCreate(smartconfig_task, "smartconfig_task", 4096, NULL, 3, NULL);
            smartconfig_done = true; // Avoid creating task multiple times
        }
    } else if (event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
        ip_event_got_ip_t* event = (ip_event_got_ip_t*) event_data;
        snprintf(current_sta_ip, sizeof(current_sta_ip), IPSTR, IP2STR(&event->ip_info.ip));
        snprintf(wifi_status_msg, sizeof(wifi_status_msg), "已连接");
        s_retry_num = 0;
        xEventGroupSetBits(s_wifi_event_group, WIFI_CONNECTED_BIT);
    } else if (event_base == SC_EVENT && event_id == SC_EVENT_GOT_SSID_PSWD) {
        smartconfig_event_got_ssid_pswd_t *evt = (smartconfig_event_got_ssid_pswd_t *)event_data;
        wifi_config_t wifi_config;
        uint8_t ssid[33] = { 0 };
        uint8_t password[65] = { 0 };

        bzero(&wifi_config, sizeof(wifi_config_t));
        memcpy(wifi_config.sta.ssid, evt->ssid, sizeof(wifi_config.sta.ssid));
        memcpy(wifi_config.sta.password, evt->password, sizeof(wifi_config.sta.password));
        wifi_config.sta.bssid_set = evt->bssid_set;
        if (wifi_config.sta.bssid_set == true) {
            memcpy(wifi_config.sta.bssid, evt->bssid, sizeof(wifi_config.sta.bssid));
        }

        memcpy(ssid, evt->ssid, sizeof(evt->ssid));
        memcpy(password, evt->password, sizeof(evt->password));
        ESP_LOGI(TAG, "SmartConfig Got SSID:%s", ssid);

        save_wifi_config((char*)ssid, (char*)password);

        ESP_ERROR_CHECK( esp_wifi_disconnect() );
        ESP_ERROR_CHECK( esp_wifi_set_config(WIFI_IF_STA, &wifi_config) );
        esp_wifi_connect();
    } else if (event_base == SC_EVENT && event_id == SC_EVENT_SEND_ACK_DONE) {
        xEventGroupSetBits(s_wifi_event_group, ESPTOUCH_DONE_BIT);
    }
}

void wifi_app_start(void) {
    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND) {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
    ESP_ERROR_CHECK(ret);

    ESP_ERROR_CHECK(esp_netif_init());
    ESP_ERROR_CHECK(esp_event_loop_create_default());
    esp_netif_create_default_wifi_sta();

    wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&cfg));

    s_wifi_event_group = xEventGroupCreate();
    ESP_ERROR_CHECK(esp_event_handler_register(WIFI_EVENT, ESP_EVENT_ANY_ID, &event_handler, NULL));
    ESP_ERROR_CHECK(esp_event_handler_register(IP_EVENT, IP_EVENT_STA_GOT_IP, &event_handler, NULL));
    ESP_ERROR_CHECK(esp_event_handler_register(SC_EVENT, ESP_EVENT_ANY_ID, &event_handler, NULL));

    char pass_conf[64] = {0};
    bool has_config = (load_wifi_config(current_sta_ssid, sizeof(current_sta_ssid), pass_conf, sizeof(pass_conf)) == ESP_OK);

    wifi_config_t wifi_config_sta = {0};
    if (has_config && strlen(current_sta_ssid) > 0) {
        strncpy((char*)wifi_config_sta.sta.ssid, current_sta_ssid, sizeof(wifi_config_sta.sta.ssid));
        strncpy((char*)wifi_config_sta.sta.password, pass_conf, sizeof(wifi_config_sta.sta.password));
    }

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    if (has_config && strlen(current_sta_ssid) > 0) {
        ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config_sta));
    }
    ESP_ERROR_CHECK(esp_wifi_start());

    initialise_mdns();

    httpd_config_t http_config = HTTPD_DEFAULT_CONFIG();
    if (httpd_start(&server, &http_config) == ESP_OK) {
        httpd_uri_t uri_api_co2 = { .uri = "/api/co2", .method = HTTP_GET, .handler = api_co2_get_handler };
        httpd_register_uri_handler(server, &uri_api_co2);

        httpd_uri_t uri_index = { .uri = "/", .method = HTTP_GET, .handler = index_get_handler };
        httpd_register_uri_handler(server, &uri_index);
    }
    
    // If no config, enter smartconfig immediately
    if (!has_config || strlen(current_sta_ssid) == 0) {
        strcpy(wifi_status_msg, "无已存配置，请使用 SmartConfig 配网");
        xTaskCreate(smartconfig_task, "smartconfig_task", 4096, NULL, 3, NULL);
        smartconfig_done = true;
    }
}
