use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub prometheus: PrometheusConfig,
    pub loki: LokiConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrometheusConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LokiConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub log_limit: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiConfig {
    pub refresh_interval_seconds: u64,
    pub log_display_count: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            prometheus: PrometheusConfig {
                base_url: "http://localhost:9090".to_string(),
                timeout_seconds: 10,
            },
            loki: LokiConfig {
                base_url: "http://localhost:3100".to_string(),
                timeout_seconds: 10,
                log_limit: 100,
            },
            ui: UiConfig {
                refresh_interval_seconds: 5,
                log_display_count: 20,
            },
        }
    }
}

