use egui::Ui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::metrics::{HistoryBuffer, SystemSnapshot};
use crate::util::format;

pub fn show(
    ui: &mut Ui,
    snapshot: Option<&SystemSnapshot>,
    memory_history: &HistoryBuffer,
    swap_history: &HistoryBuffer,
) {
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
        "Libre: {}",
        format::format_bytes(memory.free_bytes)
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

    ui.label("RAM histórica:");
    Plot::new("memory_history_plot")
        .height(120.0)
        .include_y(0.0)
        .include_y(100.0)
        .show(ui, |plot_ui| {
            let points = memory_history.rolling_points();
            plot_ui.line(Line::new(PlotPoints::new(points)).name("RAM %"));
        });

    ui.label("Swap histórico:");
    Plot::new("swap_history_plot")
        .height(120.0)
        .include_y(0.0)
        .include_y(100.0)
        .show(ui, |plot_ui| {
            let points = swap_history.rolling_points();
            plot_ui.line(Line::new(PlotPoints::new(points)).name("Swap %"));
        });
}
