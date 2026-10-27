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

    let points = cpu_history.raw_points();
    let (min_x, max_x) = if points.is_empty() {
        (0.0, 60.0)
    } else {
        let last = points[points.len() - 1][0];
        let min = (last - 60.0).max(0.0);
        let max = if last < 60.0 { 60.0 } else { last };
        (min, max)
    };

    Plot::new("cpu_history_plot")
        .height(180.0)
        .x_axis_label("Tiempo (s)")
        .y_axis_label("Uso CPU (%)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format!("{:.0}s", mark.value))
        .y_axis_formatter(|mark, _| format!("{:.0}%", mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((min_x + max_x) / 2.0, (max_x - min_x).max(1.0));
            bounds.set_y_center_height(50.0, 100.0);
            plot_ui.set_plot_bounds(bounds);
            let line_points = PlotPoints::new(points);
            plot_ui.line(Line::new(line_points).name("Global"));
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
