use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HmirConfig {
    #[serde(default = "default_port")]
    pub api_port: u16,
    #[serde(default = "default_worker_port")]
    pub worker_port: u16,
    #[serde(default = "default_refresh_ms")]
    pub telemetry_refresh_ms: u64,
    pub default_model: Option<String>,
    #[serde(default = "default_npu_priority")]
    pub npu_priority: bool,
}

fn default_port() -> u16 { 8080 }
fn default_worker_port() -> u16 { 8089 }
fn default_refresh_ms() -> u64 { 1000 }
fn default_npu_priority() -> bool { true }

impl Default for HmirConfig {
    fn default() -> Self {
        Self {
            api_port: default_port(),
            worker_port: default_worker_port(),
            telemetry_refresh_ms: default_refresh_ms(),
            default_model: None,
            npu_priority: default_npu_priority(),
        }
    }
}

impl HmirConfig {
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            toml::from_str(&content).unwrap_or_default()
        } else {
            let default = Self::default();
            let _ = default.save();
            default
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(path, content)
    }

    fn config_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hmir")
            .join("config.toml")
    }
}
