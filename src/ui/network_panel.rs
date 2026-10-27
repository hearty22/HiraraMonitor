use egui::Ui;

use crate::metrics::SystemSnapshot;
use crate::util::format;

pub fn show(
    ui: &mut Ui,
    snapshot: Option<&SystemSnapshot>,
    rx_history: &crate::metrics::HistoryBuffer,
    tx_history: &crate::metrics::HistoryBuffer,
) {
    ui.heading("Red");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    egui::Grid::new("network_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Interfaz");
            ui.label("RX/s");
            ui.label("TX/s");
            ui.label("RX total");
            ui.label("TX total");
            ui.end_row();

            for net in &snapshot.networks {
                ui.label(&net.name);
                ui.label(format::format_bytes_per_second(net.received_bytes_per_sec));
                ui.label(format::format_bytes_per_second(
                    net.transmitted_bytes_per_sec,
                ));
                ui.label(format::format_bytes(net.total_received_bytes));
                ui.label(format::format_bytes(net.total_transmitted_bytes));
                ui.end_row();
            }
        });

    use egui_plot::{Line, Plot, PlotPoints};

    fn rolling_bounds(
        history: &crate::metrics::HistoryBuffer,
        max_factor: f64,
    ) -> ([f64; 2], [f64; 2]) {
        let points = history.raw_points();
        if points.is_empty() {
            return ([0.0, 60.0], [0.0, 1.0]);
        }

        let last = points[points.len() - 1][0];
        let min_x = (last - 60.0).max(0.0);
        let max_x = if last < 60.0 { 60.0 } else { last };
        let max_y = points.iter().map(|point| point[1]).fold(1.0_f64, f64::max) * max_factor;
        ([min_x, max_x], [0.0, max_y.max(1.0)])
    }

    let (rx_x, rx_y) = rolling_bounds(rx_history, 1.15);
    ui.label("RX histórico:");
    Plot::new("net_rx_plot")
        .height(120.0)
        .x_axis_label("Tiempo (s)")
        .y_axis_label("RX (B/s)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format!("{:.0}s", mark.value))
        .y_axis_formatter(|mark, _| format::format_bytes_per_second(mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((rx_x[0] + rx_x[1]) / 2.0, (rx_x[1] - rx_x[0]).max(1.0));
            bounds.set_y_center_height((rx_y[0] + rx_y[1]) / 2.0, (rx_y[1] - rx_y[0]).max(1.0));
            plot_ui.set_plot_bounds(bounds);
            let points = PlotPoints::new(rx_history.raw_points());
            plot_ui.line(Line::new(points).name("RX"));
        });

    let (tx_x, tx_y) = rolling_bounds(tx_history, 1.15);
    ui.label("TX histórico:");
    Plot::new("net_tx_plot")
        .height(120.0)
        .x_axis_label("Tiempo (s)")
        .y_axis_label("TX (B/s)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format!("{:.0}s", mark.value))
        .y_axis_formatter(|mark, _| format::format_bytes_per_second(mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((tx_x[0] + tx_x[1]) / 2.0, (tx_x[1] - tx_x[0]).max(1.0));
            bounds.set_y_center_height((tx_y[0] + tx_y[1]) / 2.0, (tx_y[1] - tx_y[0]).max(1.0));
            plot_ui.set_plot_bounds(bounds);
            let points = PlotPoints::new(tx_history.raw_points());
            plot_ui.line(Line::new(points).name("TX"));
        });
}
