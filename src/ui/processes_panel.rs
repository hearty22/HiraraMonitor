use egui::Ui;

use crate::metrics::{ProcessSnapshot, SystemSnapshot};
use crate::ui::{ProcessSort, ProcessTableState};

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>, state: &mut ProcessTableState) {
    ui.heading("Procesos");

    ui.horizontal(|ui| {
        ui.label("Filtro:");
        let changed = ui.text_edit_singleline(&mut state.filter).changed();
        if changed {
            state.page = 0;
        }

        ui.label("Orden:");
        egui::ComboBox::from_label("sort")
            .selected_text(state.sort.label())
            .show_ui(ui, |ui| {
                for sort in [
                    ProcessSort::CpuDesc,
                    ProcessSort::MemoryDesc,
                    ProcessSort::NameAsc,
                    ProcessSort::PidAsc,
                ] {
                    ui.selectable_value(&mut state.sort, sort, sort.label());
                }
            });
    });

    let Some(snapshot) = snapshot else {
        ui.label("Esperando métricas...");
        return;
    };

    let mut filtered: Vec<&ProcessSnapshot> = snapshot
        .processes
        .iter()
        .filter(|process| {
            state.filter.trim().is_empty()
                || process
                    .name
                    .to_lowercase()
                    .contains(&state.filter.to_lowercase())
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

    egui::Grid::new("processes_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.label("PID");
            ui.label("Nombre");
            ui.label("CPU");
            ui.label("Memoria");
            ui.label("Estado");
            ui.label("Usuario");
            ui.end_row();

            for process in &filtered[start..end] {
                ui.label(process.pid.to_string());
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
}
