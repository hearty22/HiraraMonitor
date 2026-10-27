//! HiraraMonitor desktop system monitor.
//!
//! The application runs metric collection in a background thread and renders
//! the latest snapshot with egui.

mod app;
mod config;
mod config_store;
mod metrics;
mod platform;
mod ui;
mod util;

use app::HiraraApp;

fn main() -> eframe::Result<()> {
    let config = config::AppConfig::from_args();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HiraraMonitor")
            .with_inner_size([config.window_width, config.window_height]),
        ..Default::default()
    };

    eframe::run_native(
        "HiraraMonitor",
        options,
        Box::new(|_cc| Ok(Box::new(HiraraApp::new(config)))),
    )
}
