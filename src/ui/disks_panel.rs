use egui::Ui;

use crate::metrics::SystemSnapshot;
use crate::util::format;

const NAME_WIDTH: f32 = 150.0;
const MOUNT_WIDTH: f32 = 180.0;
const FS_WIDTH: f32 = 75.0;
const USED_WIDTH: f32 = 100.0;
const TOTAL_WIDTH: f32 = 100.0;
const USAGE_WIDTH: f32 = 115.0;
const RATE_WIDTH: f32 = 110.0;
const ROW_HEIGHT: f32 = 20.0;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>) {
    ui.heading("Discos");
    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new("disks_grid")
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.add_sized(
                        [NAME_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Dispositivo").strong()),
                    );
                    ui.add_sized(
                        [MOUNT_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Mount").strong()),
                    );
                    ui.add_sized(
                        [FS_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("FS").strong()),
                    );
                    ui.add_sized(
                        [USED_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Usado").strong()),
                    );
                    ui.add_sized(
                        [TOTAL_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Total").strong()),
                    );
                    ui.add_sized(
                        [USAGE_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Uso").strong()),
                    );
                    ui.add_sized(
                        [RATE_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Lectura/s").strong()),
                    );
                    ui.add_sized(
                        [RATE_WIDTH, ROW_HEIGHT],
                        egui::Label::new(egui::RichText::new("Escritura/s").strong()),
                    );
                    ui.end_row();

                    for disk in &snapshot.disks {
                        ui.add_sized(
                            [NAME_WIDTH, ROW_HEIGHT],
                            egui::Label::new(&disk.name).truncate(),
                        );
                        ui.add_sized(
                            [MOUNT_WIDTH, ROW_HEIGHT],
                            egui::Label::new(&disk.mount_point).truncate(),
                        );
                        ui.add_sized(
                            [FS_WIDTH, ROW_HEIGHT],
                            egui::Label::new(&disk.file_system).truncate(),
                        );
                        ui.add_sized(
                            [USED_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format::format_bytes(
                                disk.total_bytes.saturating_sub(disk.available_bytes),
                            )),
                        );
                        ui.add_sized(
                            [TOTAL_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format::format_bytes(disk.total_bytes)),
                        );

                        let ratio = SystemSnapshot::disk_used_percent(disk) / 100.0_f32;
                        ui.add_sized([USAGE_WIDTH, ROW_HEIGHT], egui::ProgressBar::new(ratio));

                        ui.add_sized(
                            [RATE_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format::format_bytes_per_second(
                                disk.read_bytes_per_sec,
                            )),
                        );
                        ui.add_sized(
                            [RATE_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format::format_bytes_per_second(
                                disk.write_bytes_per_sec,
                            )),
                        );
                        ui.end_row();
                    }

                    if snapshot.disks.is_empty() {
                        ui.label("No hay discos disponibles.");
                    }
                });
        });
}
