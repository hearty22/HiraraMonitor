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

    let mut sorted = snapshot.components.clone();
    sorted.sort_by(|a, b| {
        b.temperature_celsius
            .partial_cmp(&a.temperature_celsius)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ui.label("Componentes con temperatura:");
    egui::Grid::new("sensors_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Componente");
            ui.label("Actual");
            ui.label("Máxima");
            ui.label("Nivel");
            ui.end_row();

            for component in &sorted {
                let ratio = if component.max_temperature_celsius > 0.0 {
                    (component.temperature_celsius / component.max_temperature_celsius)
                        .clamp(0.0, 1.0)
                } else {
                    0.0
                };

                ui.label(&component.label);
                ui.label(format!("{:.1} °C", component.temperature_celsius));
                ui.label(format!("{:.1} °C", component.max_temperature_celsius));

                let color = if ratio >= 0.9 {
                    egui::Color32::from_rgb(220, 70, 70)
                } else if ratio >= 0.7 {
                    egui::Color32::from_rgb(220, 180, 70)
                } else {
                    egui::Color32::from_rgb(80, 180, 100)
                };
                ui.colored_label(color, format!("{:.0}%", ratio * 100.0));
                ui.end_row();
            }
        });

    ui.separator();
    ui.label("Top sensor:");
    if let Some(top) = sorted.first() {
        ui.label(format!(
            "{}: {:.1} °C / {:.1} °C",
            top.label, top.temperature_celsius, top.max_temperature_celsius
        ));
    }
}
