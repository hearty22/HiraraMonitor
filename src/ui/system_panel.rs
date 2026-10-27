use egui::Ui;

use crate::metrics::SystemSnapshot;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>) {
    ui.heading("Sistema");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    let system = &snapshot.system;
    egui::Grid::new("system_grid").striped(true).show(ui, |ui| {
        ui.label("Hostname");
        ui.label(&system.hostname);
        ui.end_row();

        ui.label("OS");
        ui.label(format!("{} {}", system.os_name, system.os_version));
        ui.end_row();

        ui.label("Kernel");
        ui.label(&system.kernel_version);
        ui.end_row();

        ui.label("Arquitectura");
        ui.label(&system.architecture);
        ui.end_row();

        ui.label("CPU count");
        ui.label(system.cpu_count.to_string());
        ui.end_row();

        ui.label("Uptime");
        ui.label(format!("{} s", system.uptime_seconds));
        ui.end_row();

        ui.label("Load avg");
        ui.label(format!(
            "{:.2} / {:.2} / {:.2}",
            system.load_average_one, system.load_average_five, system.load_average_fifteen
        ));
        ui.end_row();
    });
}
