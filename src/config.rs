use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_millis(1000),
            history_len: 300,
            process_page_size: 25,
            theme: Theme::Dark,
            initial_view: View::Cpu,
        }
    }
}

impl AppConfig {
    pub fn from_args() -> Self {
        let mut config = Self::default();
        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--refresh-ms" => {
                    if let Some(value) = args.get(i + 1) {
                        if let Ok(ms) = value.parse::<u64>() {
                            config.refresh_interval = Duration::from_millis(ms.max(250));
                        }
                        i += 2;
                        continue;
                    }
                }
                "--history" => {
                    if let Some(value) = args.get(i + 1) {
                        if let Ok(len) = value.parse::<usize>() {
                            config.history_len = len.clamp(30, 5000);
                        }
                        i += 2;
                        continue;
                    }
                }
                "--light" => config.theme = Theme::Light,
                "--dark" => config.theme = Theme::Dark,
                _ => {}
            }
            i += 1;
        }
        config
    }
}
