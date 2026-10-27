use egui::Ui;

use crate::metrics::SystemSnapshot;
use crate::util::format;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>) {
    ui.heading("Discos");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    egui::Grid::new("disks_grid").striped(true).show(ui, |ui| {
        ui.label("Dispositivo");
        ui.label("Mount");
        ui.label("Usado");
        ui.label("Total");
        ui.label("Gauge");
        ui.label("Lectura/s");
        ui.label("Escritura/s");
        ui.end_row();

        for disk in &snapshot.disks {
            ui.label(&disk.name);
            ui.label(&disk.mount_point);
            ui.label(format::format_bytes(
                disk.total_bytes.saturating_sub(disk.available_bytes),
            ));
            ui.label(format::format_bytes(disk.total_bytes));
            ui.label(format!(
                "{:.1}%",
                if disk.total_bytes == 0 {
                    0.0
                } else {
                    (disk.total_bytes.saturating_sub(disk.available_bytes)) as f64
                        / disk.total_bytes as f64
                        * 100.0
                }
            ));
            ui.label(format::format_bytes_per_second(disk.read_bytes_per_sec));
            ui.label(format::format_bytes_per_second(disk.write_bytes_per_sec));
            ui.end_row();
        }
    });
}
