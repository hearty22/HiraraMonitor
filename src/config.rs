use crate::ui::ProcessSort;
use std::time::Duration;

use crate::config_store::ConfigStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    #[default]
    Cpu,
    Memory,
    Disks,
    Network,
    Processes,
    Sensors,
    System,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub refresh_interval: Duration,
    pub history_len: usize,
    pub process_page_size: usize,
    pub theme: Theme,
    pub initial_view: View,
    pub process_sort: ProcessSort,
    pub visual_alerts: bool,
    pub cpu_warn: f32,
    pub cpu_crit: f32,
    pub mem_warn: f32,
    pub mem_crit: f32,
    pub window_width: f32,
    pub window_height: f32,
    pub config_path: Option<std::path::PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_millis(default_refresh_interval_ms()),
            history_len: default_history_len(),
            process_page_size: default_process_page_size(),
            theme: Theme::Dark,
            initial_view: View::Cpu,
            process_sort: ProcessSort::CpuDesc,
            visual_alerts: true,
            cpu_warn: 70.0,
            cpu_crit: 90.0,
            mem_warn: 80.0,
            mem_crit: 95.0,
            window_width: default_window_width(),
            window_height: default_window_height(),
            config_path: None,
        }
    }
}

impl AppConfig {
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut config_path: Option<std::path::PathBuf> = None;
        let mut refresh_override: Option<u64> = None;
        let mut history_override: Option<usize> = None;
        let mut theme_override: Option<Theme> = None;
        let initial_view_override: Option<View> = None;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--config" => {
                    if let Some(value) = args.get(i + 1) {
                        config_path = Some(std::path::PathBuf::from(value));
                        i += 2;
                        continue;
                    }
                }
                "--refresh-ms" => {
                    if let Some(value) = args.get(i + 1) {
                        if let Ok(ms) = value.parse::<u64>() {
                            refresh_override = Some(ms.max(250));
                        }
                        i += 2;
                        continue;
                    }
                }
                "--history" => {
                    if let Some(value) = args.get(i + 1) {
                        if let Ok(len) = value.parse::<usize>() {
                            history_override = Some(len.clamp(30, 5000));
                        }
                        i += 2;
                        continue;
                    }
                }
                "--light" => {
                    theme_override = Some(Theme::Light);
                }
                "--dark" => {
                    theme_override = Some(Theme::Dark);
                }
                _ => {}
            }
            i += 1;
        }

        let environment_path = std::env::var_os("HIRARA_CONFIG").map(std::path::PathBuf::from);
        let resolved_path = config_path
            .clone()
            .or(environment_path)
            .or_else(ConfigStore::default_path);

        let store = ConfigStore::new(resolved_path.clone());
        let mut config = store.load_or_default();

        if let Some(ms) = refresh_override {
            config.refresh_interval = Duration::from_millis(ms);
        }
        if let Some(len) = history_override {
            config.history_len = len;
        }
        if let Some(theme) = theme_override {
            config.theme = theme;
        }
        if let Some(view) = initial_view_override {
            config.initial_view = view;
        }

        config.config_path = resolved_path;
        config
    }

    pub fn save(&self) -> Result<(), String> {
        let store = ConfigStore::new(self.config_path.clone());
        store
            .save(&PersistedConfig::from(self))
            .map_err(|error| error.to_string())
    }
}

fn default_refresh_interval_ms() -> u64 {
    1000
}

fn default_history_len() -> usize {
    300
}

fn default_process_page_size() -> usize {
    25
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedConfig {
    pub theme: Theme,
    pub initial_view: View,
    #[serde(default = "default_refresh_interval_ms")]
    pub refresh_interval_ms: u64,
    #[serde(default = "default_history_len")]
    pub history_len: usize,
    #[serde(default = "default_process_page_size")]
    pub process_page_size: usize,
    #[serde(default)]
    pub process_sort: ProcessSort,
    #[serde(default = "default_true")]
    pub visual_alerts: bool,
    #[serde(default = "default_cpu_warn")]
    pub cpu_warn: f32,
    #[serde(default = "default_cpu_crit")]
    pub cpu_crit: f32,
    #[serde(default = "default_mem_warn")]
    pub mem_warn: f32,
    #[serde(default = "default_mem_crit")]
    pub mem_crit: f32,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
}

impl Default for PersistedConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            initial_view: View::Cpu,
            refresh_interval_ms: default_refresh_interval_ms(),
            history_len: default_history_len(),
            process_page_size: default_process_page_size(),
            process_sort: ProcessSort::CpuDesc,
            visual_alerts: true,
            cpu_warn: default_cpu_warn(),
            cpu_crit: default_cpu_crit(),
            mem_warn: default_mem_warn(),
            mem_crit: default_mem_crit(),
            window_width: default_window_width(),
            window_height: default_window_height(),
        }
    }
}

impl PersistedConfig {
    pub fn from(config: &AppConfig) -> Self {
        Self {
            theme: config.theme,
            initial_view: config.initial_view,
            refresh_interval_ms: config.refresh_interval.as_millis().min(u64::MAX as u128) as u64,
            history_len: config.history_len,
            process_page_size: config.process_page_size,
            process_sort: config.process_sort,
            visual_alerts: config.visual_alerts,
            cpu_warn: config.cpu_warn,
            cpu_crit: config.cpu_crit,
            mem_warn: config.mem_warn,
            mem_crit: config.mem_crit,
            window_width: config.window_width,
            window_height: config.window_height,
        }
    }
}

impl AppConfig {
    pub fn from_persisted(
        persisted: PersistedConfig,
        config_path: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            refresh_interval: Duration::from_millis(
                persisted
                    .refresh_interval_ms
                    .max(default_refresh_interval_ms()),
            ),
            history_len: persisted.history_len.clamp(30, 5000),
            process_page_size: persisted.process_page_size.clamp(5, 200),
            theme: persisted.theme,
            initial_view: persisted.initial_view,
            process_sort: persisted.process_sort,
            visual_alerts: persisted.visual_alerts,
            cpu_warn: persisted.cpu_warn,
            cpu_crit: persisted.cpu_crit,
            mem_warn: persisted.mem_warn,
            mem_crit: persisted.mem_crit,
            window_width: if persisted.window_width > 0.0 {
                persisted.window_width
            } else {
                default_window_width()
            },
            window_height: if persisted.window_height > 0.0 {
                persisted.window_height
            } else {
                default_window_height()
            },
            config_path,
        }
    }
}

fn default_window_width() -> f32 {
    1200.0
}

fn default_window_height() -> f32 {
    760.0
}

fn default_true() -> bool {
    true
}

fn default_cpu_warn() -> f32 {
    70.0
}

fn default_cpu_crit() -> f32 {
    90.0
}

fn default_mem_warn() -> f32 {
    80.0
}

fn default_mem_crit() -> f32 {
    95.0
}
