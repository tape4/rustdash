use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
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

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(
                Environment::with_prefix("MONITOR")
                    .separator("_")
                    .try_parsing(true),
            )
            .set_default("prometheus.base_url", "http://localhost:9090")?
            .set_default("prometheus.timeout_seconds", 10)?
            .set_default("loki.base_url", "http://localhost:3100")?
            .set_default("loki.timeout_seconds", 10)?
            .set_default("loki.log_limit", 100)?
            .set_default("ui.refresh_interval_seconds", 5)?
            .set_default("ui.log_display_count", 20)?
            .build()?;

        settings.try_deserialize()
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(
                Environment::with_prefix("MONITOR")
                    .separator("_")
                    .try_parsing(true),
            )
            .set_default("prometheus.base_url", "http://localhost:9090")?
            .set_default("prometheus.timeout_seconds", 10)?
            .set_default("loki.base_url", "http://localhost:3100")?
            .set_default("loki.timeout_seconds", 10)?
            .set_default("loki.log_limit", 100)?
            .set_default("ui.refresh_interval_seconds", 5)?
            .set_default("ui.log_display_count", 20)?
            .build()?;

        settings.try_deserialize()
    }
}