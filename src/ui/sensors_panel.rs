use egui::Ui;

use crate::metrics::SystemSnapshot;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>) {
    ui.heading("Sensores");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    if snapshot.components.is_empty() {
        ui.label("No hay sensores disponibles a través de sysinfo.");
        return;
    }

    egui::Grid::new("sensors_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Componente");
            ui.label("Temp actual");
            ui.label("Temp máxima");
            ui.end_row();

            for component in &snapshot.components {
                ui.label(&component.label);
                ui.label(format!("{:.1} °C", component.temperature_celsius));
                ui.label(format!("{:.1} °C", component.max_temperature_celsius));
                ui.end_row();
            }
        });
}
