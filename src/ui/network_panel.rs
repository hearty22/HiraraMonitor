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

    let (global_rx, global_tx): (f64, f64) =
        snapshot.networks.iter().fold((0.0, 0.0), |acc, net| {
            (
                acc.0 + net.received_bytes_per_sec,
                acc.1 + net.transmitted_bytes_per_sec,
            )
        });

    ui.label("Resumen horizontal:");
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                interface_card(
                    ui,
                    "Total",
                    "Agregado",
                    global_rx,
                    global_tx,
                    snapshot
                        .networks
                        .iter()
                        .map(|n| n.total_received_bytes)
                        .sum(),
                    snapshot
                        .networks
                        .iter()
                        .map(|n| n.total_transmitted_bytes)
                        .sum(),
                );

                for net in &snapshot.networks {
                    interface_card(
                        ui,
                        &net.name,
                        "Interfaz",
                        net.received_bytes_per_sec,
                        net.transmitted_bytes_per_sec,
                        net.total_received_bytes,
                        net.total_transmitted_bytes,
                    );
                }
            });
        });

    ui.separator();
    use crate::metrics::HistoryBuffer;
    use egui_plot::{Line, Plot, PlotPoints};

    fn rolling_bounds(history: &HistoryBuffer) -> ([f64; 2], f64) {
        let points = history.raw_points();
        if points.is_empty() {
            return ([0.0, 60.0], 1.0);
        }

        let last = points[points.len() - 1][0];
        let min_x = (last - 60.0).max(0.0);
        let max_x = if last < 60.0 { 60.0 } else { last };
        let max_data = points.iter().map(|point| point[1]).fold(1.0, f64::max);
        ([min_x, max_x], max_data * 1.15)
    }

    let (rx_x, rx_y) = rolling_bounds(rx_history);
    ui.label("RX histórico:");
    Plot::new("net_rx_plot")
        .height(110.0)
        .x_axis_label("Tiempo (s)")
        .y_axis_label("RX (B/s)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format!("{:.0}s", mark.value))
        .y_axis_formatter(|mark, _| format::format_bytes_per_second(mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((rx_x[0] + rx_x[1]) / 2.0, (rx_x[1] - rx_x[0]).max(1.0));
            bounds.set_y_center_height(rx_y / 2.0, rx_y);
            plot_ui.set_plot_bounds(bounds);
            let points = PlotPoints::new(rx_history.raw_points());
            plot_ui.line(Line::new(points).name("RX"));
        });

    let (tx_x, tx_y) = rolling_bounds(tx_history);
    ui.label("TX histórico:");
    Plot::new("net_tx_plot")
        .height(110.0)
        .x_axis_label("Tiempo (s)")
        .y_axis_label("TX (B/s)")
        .allow_drag(false)
        .allow_zoom(false)
        .x_axis_formatter(|mark, _| format!("{:.0}s", mark.value))
        .y_axis_formatter(|mark, _| format::format_bytes_per_second(mark.value))
        .show(ui, |plot_ui| {
            let mut bounds = plot_ui.plot_bounds();
            bounds.set_x_center_width((tx_x[0] + tx_x[1]) / 2.0, (tx_x[1] - tx_x[0]).max(1.0));
            bounds.set_y_center_height(tx_y / 2.0, tx_y);
            plot_ui.set_plot_bounds(bounds);
            let points = PlotPoints::new(tx_history.raw_points());
            plot_ui.line(Line::new(points).name("TX"));
        });
}

fn interface_card(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    rx: f64,
    tx: f64,
    total_rx: u64,
    total_tx: u64,
) {
    egui::Frame::group(ui.style())
        .rounding(egui::Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_min_width(190.0);
            ui.vertical(|ui| {
                ui.strong(title);
                ui.label(subtitle);
                ui.label(format!("RX: {}", format::format_bytes_per_second(rx)));
                ui.label(format!("TX: {}", format::format_bytes_per_second(tx)));
                ui.label(format!("RX total: {}", format::format_bytes(total_rx)));
                ui.label(format!("TX total: {}", format::format_bytes(total_tx)));
            });
        });
    ui.add_space(8.0);
}
