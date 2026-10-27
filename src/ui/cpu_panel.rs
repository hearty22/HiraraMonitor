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

    if let Some(freq) = snapshot.cpu_frequencies_mhz.first() {
        ui.label(format!("Frecuencia del primer núcleo: {} MHz", freq));
    }

    ui.label(format!("Núcleos: {}", snapshot.cpu_cores.len()));

    Plot::new("cpu_history_plot")
        .height(180.0)
        .show(ui, |plot_ui| {
            let points: PlotPoints = cpu_history.points().iter().map(|(x, y)| [*x, *y]).collect();
            plot_ui.line(Line::new(points).name("Global"));
        });

    ui.separator();
    ui.label("Por núcleo:");

    egui::Grid::new("cpu_cores_grid")
        .striped(true)
        .show(ui, |ui| {
            for (index, usage) in snapshot.cpu_cores.iter().enumerate() {
                ui.label(format!("Core {}", index));
                ui.label(format::format_percent(*usage));
                ui.end_row();
            }
        });
}
