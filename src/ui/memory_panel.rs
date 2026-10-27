use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::metrics::{HistoryBuffer, SystemSnapshot};
use crate::util::format;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>, memory_history: &HistoryBuffer) {
    ui.heading("Memoria");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    let memory = &snapshot.memory;
    ui.label(format!(
        "RAM: {} / {}",
        format::format_bytes(memory.used_bytes),
        format::format_bytes(memory.total_bytes)
    ));
    ui.label(format!(
        "Disponible: {}",
        format::format_bytes(memory.available_bytes)
    ));
    ui.label(format!(
        "Uso: {}",
        format::format_percent(snapshot.memory_used_percent())
    ));

    if memory.swap_total_bytes > 0 {
        ui.label(format!(
            "Swap: {} / {}",
            format::format_bytes(memory.swap_used_bytes),
            format::format_bytes(memory.swap_total_bytes)
        ));
    }

    Plot::new("memory_history_plot")
        .height(180.0)
        .show(ui, |plot_ui| {
            let points: PlotPoints = memory_history
                .points()
                .iter()
                .map(|(x, y)| [*x, *y])
                .collect();
            plot_ui.line(Line::new(points).name("RAM %"));
        });
}
