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

    Plot::new("net_rx_plot").height(120.0).show(ui, |plot_ui| {
        let points = rx_history.rolling_points();
        plot_ui.line(Line::new(PlotPoints::new(points)).name("RX"));
    });

    Plot::new("net_tx_plot").height(120.0).show(ui, |plot_ui| {
        let points = tx_history.rolling_points();
        plot_ui.line(Line::new(PlotPoints::new(points)).name("TX"));
    });
}
