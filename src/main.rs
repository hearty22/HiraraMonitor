mod app;
mod config;
mod metrics;
mod platform;
mod ui;
mod util;

use app::HiraraApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HiraraMonitor")
            .with_inner_size([1200.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "HiraraMonitor",
        options,
        Box::new(|_cc| Ok(Box::new(HiraraApp::new()))),
    )
}
