use egui::Ui;

use crate::metrics::{ProcessSnapshot, SystemSnapshot};
use crate::ui::{ProcessSort, ProcessTableState};

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>, state: &mut ProcessTableState) {
    ui.heading("Procesos");

    ui.horizontal(|ui| {
        ui.label("Filtro:");
        if ui.text_edit_singleline(&mut state.filter).changed() {
            state.page = 0;
        }

        ui.label("Orden:");
        egui::ComboBox::from_label("sort")
            .selected_text(state.sort.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut state.sort,
                    ProcessSort::CpuDesc,
                    ProcessSort::CpuDesc.label(),
                );
                ui.selectable_value(
                    &mut state.sort,
                    ProcessSort::MemoryDesc,
                    ProcessSort::MemoryDesc.label(),
                );
                ui.selectable_value(
                    &mut state.sort,
                    ProcessSort::NameAsc,
                    ProcessSort::NameAsc.label(),
                );
                ui.selectable_value(
                    &mut state.sort,
                    ProcessSort::PidAsc,
                    ProcessSort::PidAsc.label(),
                );
            });
    });

    ui.separator();

    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    let mut filtered: Vec<&ProcessSnapshot> = snapshot
        .processes
        .iter()
        .filter(|process| {
            if state.filter.trim().is_empty() {
                true
            } else {
                let needle = state.filter.to_lowercase();
                process.name.to_lowercase().contains(&needle)
                    || process.pid.to_string().contains(&needle)
                    || process.user.to_lowercase().contains(&needle)
            }
        })
        .collect();

    filtered.sort_by(|a, b| match state.sort {
        ProcessSort::CpuDesc => b
            .cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal),
        ProcessSort::MemoryDesc => b.memory_bytes.cmp(&a.memory_bytes),
        ProcessSort::NameAsc => a.name.cmp(&b.name),
        ProcessSort::PidAsc => a.pid.cmp(&b.pid),
    });

    ui.label(format!(
        "Top CPU: {}",
        top_name(&filtered, ProcessSort::CpuDesc)
    ));
    ui.label(format!(
        "Top memoria: {}",
        top_name(&filtered, ProcessSort::MemoryDesc)
    ));

    let total_pages = if filtered.is_empty() {
        1
    } else {
        (filtered.len() + state.page_size - 1) / state.page_size
    };
    if state.page >= total_pages {
        state.page = total_pages.saturating_sub(1);
    }

    let start = state.page * state.page_size;
    let end = (start + state.page_size).min(filtered.len());

    egui::ScrollArea::vertical()
        .max_height(260.0)
        .show(ui, |ui| {
            egui::Grid::new("processes_grid")
                .striped(true)
                .show(ui, |ui| {
                    if ui.button("PID").clicked() {
                        state.sort = ProcessSort::PidAsc;
                    }
                    if ui.button("Nombre").clicked() {
                        state.sort = ProcessSort::NameAsc;
                    }
                    if ui.button("CPU").clicked() {
                        state.sort = ProcessSort::CpuDesc;
                    }
                    if ui.button("Memoria").clicked() {
                        state.sort = ProcessSort::MemoryDesc;
                    }
                    ui.label("Estado");
                    ui.label("Usuario");
                    ui.end_row();

                    for process in &filtered[start..end] {
                        let selected = state.selected_pid == Some(process.pid);
                        if ui
                            .selectable_label(selected, process.pid.to_string())
                            .clicked()
                        {
                            state.selected_pid = Some(process.pid);
                        }
                        ui.label(&process.name);
                        ui.label(format!("{:.1}%", process.cpu_usage));
                        ui.label(format!(
                            "{:.1} MB",
                            process.memory_bytes as f64 / 1024.0 / 1024.0
                        ));
                        ui.label(&process.state);
                        ui.label(&process.user);
                        ui.end_row();
                    }
                });
        });

    ui.horizontal(|ui| {
        if ui.button("Anterior").clicked() && state.page > 0 {
            state.page -= 1;
        }
        ui.label(format!("Página {} / {}", state.page + 1, total_pages));
        if ui.button("Siguiente").clicked() && state.page + 1 < total_pages {
            state.page += 1;
        }
        ui.label(format!("Total: {}", filtered.len()));
    });

    if let Some(selected_pid) = state.selected_pid {
        if let Some(process) = snapshot.processes.iter().find(|p| p.pid == selected_pid) {
            ui.separator();
            ui.heading("Detalle del proceso");
            egui::Grid::new("process_detail_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("PID");
                    ui.label(process.pid.to_string());
                    ui.end_row();

                    if let Some(parent_pid) = process.parent_pid {
                        ui.label("PPID");
                        ui.label(parent_pid.to_string());
                        ui.end_row();
                    }

                    ui.label("Nombre");
                    ui.label(&process.name);
                    ui.end_row();

                    ui.label("CPU");
                    ui.label(format!("{:.1}%", process.cpu_usage));
                    ui.end_row();

                    ui.label("Memoria");
                    ui.label(format!(
                        "{:.1} MB",
                        process.memory_bytes as f64 / 1024.0 / 1024.0
                    ));
                    ui.end_row();

                    ui.label("Estado");
                    ui.label(&process.state);
                    ui.end_row();

                    ui.label("Usuario");
                    ui.label(&process.user);
                    ui.end_row();

                    if !process.executable_path.is_empty() {
                        ui.label("Executable");
                        ui.label(&process.executable_path);
                        ui.end_row();
                    }

                    if !process.command.is_empty() {
                        ui.label("Comando");
                        ui.label(process.command.join(" "));
                        ui.end_row();
                    }
                });
        } else {
            state.selected_pid = None;
        }
    }
}

fn top_name(filtered: &[&ProcessSnapshot], sort: ProcessSort) -> String {
    let mut candidates = filtered.to_vec();
    candidates.sort_by(|a, b| match sort {
        ProcessSort::CpuDesc => b
            .cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal),
        ProcessSort::MemoryDesc => b.memory_bytes.cmp(&a.memory_bytes),
        _ => std::cmp::Ordering::Equal,
    });

    candidates
        .first()
        .map(|process| format!("{} ({:.1}%)", process.name, process.cpu_usage))
        .unwrap_or_else(|| "-".to_string())
}
