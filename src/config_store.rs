use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{AppConfig, PersistedConfig};

/// Loads and stores TOML application preferences.
/// Loads and stores TOML application preferences.
pub struct ConfigStore {
    path: Option<PathBuf>,
}

impl ConfigStore {
    /// Creates a new store bound to an optional file path.
    /// Creates a new store bound to an optional file path.
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path }
    }

    /// Returns the default Linux config path from `HOME`.
    /// Returns the default Linux config path from `HOME`.
    pub fn default_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from).map(|home| {
            home.join(".config")
                .join("hiraramonitor")
                .join("config.toml")
        })
    }

    #[allow(dead_code)]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    #[allow(dead_code)]
    /// Loads the persisted file, returning defaults on any read or parse error.
    /// Loads the persisted file, returning defaults on any read or parse error.
    pub fn load(&self) -> PersistedConfig {
        self.load_or_default_lossy()
    }

    /// Loads config or applies fallback validation.
    /// Loads config or applies fallback validation.
    pub fn load_or_default(&self) -> AppConfig {
        let persisted = self.load_or_default_lossy();
        let normalized = AppConfig::from_persisted(persisted, self.path.clone());

        #[cfg(test)]
        let _ = &normalized;

        normalized
    }

    fn load_or_default_lossy(&self) -> PersistedConfig {
        let Some(path) = &self.path else {
            return PersistedConfig::default();
        };
        if !path.is_file() {
            return PersistedConfig::default();
        }
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(_) => return PersistedConfig::default(),
        };
        let mut persisted: PersistedConfig = toml::from_str(&contents).unwrap_or_default();
        if persisted.window_width <= 0.0 {
            persisted.window_width = 1200.0;
        }
        if persisted.window_height <= 0.0 {
            persisted.window_height = 760.0;
        }
        persisted
    }

    /// Atomically writes configuration to the target path.
    /// Atomically writes configuration to the target path.
    pub fn save(&self, config: &PersistedConfig) -> std::io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut contents = toml::to_string_pretty(config)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        if !contents.ends_with('\n') {
            contents.push('\n');
        }

        let temporary_path = path.with_extension("toml.tmp");
        fs::write(&temporary_path, contents)?;
        fs::rename(&temporary_path, path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigStore;
    use crate::config::PersistedConfig;
    use std::fs;

    #[test]
    fn round_trips_persisted_config() {
        let path = std::env::temp_dir().join("hirara-monitor-config-roundtrip.toml");
        let store = ConfigStore::new(Some(path.clone()));
        let config = PersistedConfig {
            theme: crate::config::Theme::Light,
            initial_view: crate::config::View::Processes,
            refresh_interval_ms: 500,
            history_len: 120,
            process_page_size: 12,
            process_sort: crate::ui::ProcessSort::MemoryDesc,
            visual_alerts: true,
            cpu_warn: 60.0,
            cpu_crit: 85.0,
            mem_warn: 75.0,
            mem_crit: 90.0,
            window_width: 900.0,
            window_height: 600.0,
        };

        store.save(&config).expect("save config");
        let loaded = store.load();

        assert_eq!(config, loaded);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_is_default() {
        let path = std::env::temp_dir().join("hirara-monitor-missing-config-target.toml");
        let _ = fs::remove_file(&path);
        let loaded = ConfigStore::new(Some(path)).load();
        assert_eq!(loaded, PersistedConfig::default());
    }
}
