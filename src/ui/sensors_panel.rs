use egui::Ui;

use crate::metrics::SystemSnapshot;

const COMPONENT_WIDTH: f32 = 280.0;
const TEMP_WIDTH: f32 = 100.0;
const LEVEL_WIDTH: f32 = 120.0;
const ROW_HEIGHT: f32 = 20.0;

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

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new("sensors_grid")
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.add_sized(
                        [COMPONENT_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Componente").strong()),
                    );
                    ui.add_sized(
                        [TEMP_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Actual").strong()),
                    );
                    ui.add_sized(
                        [TEMP_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Máxima").strong()),
                    );
                    ui.add_sized(
                        [LEVEL_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Nivel").strong()),
                    );
                    ui.end_row();

                    for component in &sorted {
                        let ratio = if component.max_temperature_celsius > 0.0 {
                            (component.temperature_celsius / component.max_temperature_celsius)
                                .clamp(0.0, 1.0)
                        } else {
                            0.0
                        };

                        ui.add_sized(
                            [COMPONENT_WIDTH, ROW_HEIGHT],
                            egui::Label::new(&component.label).truncate(),
                        );
                        ui.add_sized(
                            [TEMP_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format!("{:.1} °C", component.temperature_celsius)),
                        );
                        ui.add_sized(
                            [TEMP_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format!(
                                "{:.1} °C",
                                component.max_temperature_celsius
                            )),
                        );

                        let color = if ratio >= 0.9 {
                            egui::Color32::from_rgb(220, 70, 70)
                        } else if ratio >= 0.7 {
                            egui::Color32::from_rgb(220, 180, 70)
                        } else {
                            egui::Color32::from_rgb(80, 180, 100)
                        };
                        ui.add_sized(
                            [LEVEL_WIDTH, ROW_HEIGHT],
                            egui::Label::new(
                                egui::RichText::new(format!("{:.0}%", ratio * 100.0)).color(color),
                            ),
                        );
                        ui.end_row();
                    }
                });
        });

    if let Some(top) = sorted.first() {
        ui.separator();
        ui.label(format!(
            "Sensor más alto: {} — {:.1} °C / {:.1} °C",
            top.label, top.temperature_celsius, top.max_temperature_celsius
        ));
    }
}
