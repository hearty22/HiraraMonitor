use egui::Ui;

use crate::metrics::SystemSnapshot;

const LABEL_WIDTH: f32 = 150.0;
const VALUE_WIDTH: f32 = 320.0;
const ROW_HEIGHT: f32 = 22.0;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>) {
    ui.heading("Sistema");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    let system = &snapshot.system;

    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        egui::Grid::new("system_grid")
            .striped(true)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                row(ui, "Hostname", &system.hostname);
                row(
                    ui,
                    "OS",
                    format!("{} {}", system.os_name, system.os_version).as_str(),
                );
                row(ui, "Kernel", &system.kernel_version);
                row(ui, "Arquitectura", &system.architecture);
                row(ui, "Cantidad de CPU", &system.cpu_count.to_string());
                row(ui, "Uptime", &format!("{} s", system.uptime_seconds));
                row(
                    ui,
                    "Load average",
                    format!(
                        "{:.2} / {:.2} / {:.2}",
                        system.load_average_one,
                        system.load_average_five,
                        system.load_average_fifteen
                    )
                    .as_str(),
                );
            });
    });

    ui.add_space(8.0);
    ui.group(|ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            info_card(
                ui,
                "CPU",
                &format!("{} núcleos", system.cpu_count),
                egui::Color32::from_rgb(70, 130, 210),
            );
            info_card(
                ui,
                "RAM",
                "Monitoreo activo",
                egui::Color32::from_rgb(80, 180, 100),
            );
            info_card(
                ui,
                "Uptime",
                &format!("{} s", system.uptime_seconds),
                egui::Color32::from_rgb(210, 130, 70),
            );
            info_card(
                ui,
                "Load",
                &format!("{:.2}", system.load_average_one),
                egui::Color32::from_rgb(180, 100, 210),
            );
        });
    });
}

fn row(ui: &mut Ui, label: &str, value: &str) {
    ui.add_sized(
        [LABEL_WIDTH, ROW_HEIGHT],
        egui::Label::new(egui::RichText::new(label).strong()),
    );
    ui.add_sized(
        [VALUE_WIDTH, ROW_HEIGHT],
        egui::Label::new(value).truncate(),
    );
    ui.end_row();
}

fn info_card(ui: &mut Ui, title: &str, value: &str, color: egui::Color32) {
    egui::Frame::group(ui.style())
        .fill(color.linear_multiply(0.18))
        .stroke(egui::Stroke::new(1.0, color))
        .rounding(egui::Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_min_width(140.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).strong().color(color));
                ui.label(egui::RichText::new(value).size(16.0));
            });
        });
    ui.add_space(8.0);
}
