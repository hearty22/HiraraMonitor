use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::metrics::{HistoryBuffer, SystemSnapshot};
use crate::util::format;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>, cpu_history: &HistoryBuffer) {
    ui.heading("CPU");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    ui.label(format!(
        "Uso global: {}",
        format::format_percent(snapshot.cpu_global)
    ));

    if let Some(temperature) = snapshot.cpu_temperature_celsius {
        ui.label(format!("Temperatura CPU: {:.1} °C", temperature));
    }

    if let Some(core) = snapshot.cpu_cores.first() {
        ui.label(format!(
            "Frecuencia del primer núcleo: {} MHz",
            core.frequency_mhz
        ));
    }

    ui.label(format!("Núcleos: {}", snapshot.cpu_cores.len()));

    Plot::new("cpu_history_plot")
        .height(180.0)
        .include_y(0.0)
        .include_y(100.0)
        .show(ui, |plot_ui| {
            let points = cpu_history.rolling_points();
            plot_ui.line(Line::new(PlotPoints::new(points)).name("Global"));
        });

    ui.separator();
    ui.label("Por núcleo:");

    egui::Grid::new("cpu_cores_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Núcleo");
            ui.label("Uso");
            ui.label("Frecuencia");
            ui.label("Temperatura");
            ui.end_row();

            for (index, core) in snapshot.cpu_cores.iter().enumerate() {
                ui.label(format!("Core {}", index));
                ui.label(format::format_percent(core.usage));
                ui.label(format!("{} MHz", core.frequency_mhz));
                ui.label(
                    core.temperature_celsius
                        .map(|temperature| format!("{:.1} °C", temperature))
                        .unwrap_or_else(|| "-".to_string()),
                );
                ui.end_row();
            }
        });
}
