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

    let memory_points = memory_history.raw_points();
    let (mem_min, mem_max) = if memory_points.is_empty() {
        (0.0, 60.0)
    } else {
        let last = memory_points[memory_points.len() - 1][0];
        let min = (last - 60.0).max(0.0);
        let max = if last < 60.0 { 60.0 } else { last };
        (min, max)
    };

    ui.label("RAM histórica:");
    Plot::new("memory_history_plot")
        .height(120.0)
        .x_axis_label("Hora")
        .y_axis_label("Uso RAM (%)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format::format_epoch_time(mark.value))
        .y_axis_formatter(|mark, _| format!("{:.0}%", mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((mem_min + mem_max) / 2.0, (mem_max - mem_min).max(1.0));
            bounds.set_y_center_height(50.0, 100.0);
            plot_ui.set_plot_bounds(bounds);
            let line_points = PlotPoints::new(memory_points);
            plot_ui.line(Line::new(line_points).name("RAM %"));
        });

    let swap_points = swap_history.raw_points();
    let (swap_min, swap_max) = if swap_points.is_empty() {
        (0.0, 60.0)
    } else {
        let last = swap_points[swap_points.len() - 1][0];
        let min = (last - 60.0).max(0.0);
        let max = if last < 60.0 { 60.0 } else { last };
        (min, max)
    };

    ui.label("Swap histórico:");
    Plot::new("swap_history_plot")
        .height(120.0)
        .x_axis_label("Hora")
        .y_axis_label("Uso swap (%)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format::format_epoch_time(mark.value))
        .y_axis_formatter(|mark, _| format!("{:.0}%", mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((swap_min + swap_max) / 2.0, (swap_max - swap_min).max(1.0));
            bounds.set_y_center_height(50.0, 100.0);
            plot_ui.set_plot_bounds(bounds);
            let line_points = PlotPoints::new(swap_points);
            plot_ui.line(Line::new(line_points).name("Swap %"));
        });
}
