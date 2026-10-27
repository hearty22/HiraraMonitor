use egui::Ui;

use crate::metrics::{ProcessSnapshot, SystemSnapshot};
use crate::platform;
use crate::ui::{
    PendingProcessAction, ProcessActionKind, ProcessSort, ProcessTableState,
};

const PID_WIDTH: f32 = 70.0;
const CPU_WIDTH: f32 = 80.0;
const MEMORY_WIDTH: f32 = 110.0;
const STATE_WIDTH: f32 = 95.0;
const USER_WIDTH: f32 = 120.0;
const ROW_HEIGHT: f32 = 18.0;

pub fn show(ui: &mut Ui, snapshot: Option<&SystemSnapshot>, state: &mut ProcessTableState) {
    ui.heading("Procesos");

    if let Some((message, is_error)) = state.action_status.clone() {
        let color = if is_error {
            egui::Color32::from_rgb(245, 105, 85)
        } else {
            egui::Color32::from_rgb(95, 185, 110)
        };
        ui.colored_label(color, message);
        if ui.button("Cerrar").clicked() {
            state.action_status = None;
        }
    }

    let name_width =
        (ui.available_width() - PID_WIDTH - CPU_WIDTH - MEMORY_WIDTH - STATE_WIDTH - USER_WIDTH)
            .max(120.0);

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
                    || process.command.join(" ").to_lowercase().contains(&needle)
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
        "Top CPU: {}  |  Top memoria: {}",
        top_name(&filtered, ProcessSort::CpuDesc),
        top_name(&filtered, ProcessSort::MemoryDesc)
    ));

    let total_pages = if filtered.is_empty() {
        1
    } else {
        filtered.len().div_ceil(state.page_size)
    };
    if state.page >= total_pages {
        state.page = total_pages.saturating_sub(1);
    }

    let start = state.page * state.page_size;
    let end = (start + state.page_size).min(filtered.len());

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

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new("processes_grid")
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    if ui
                        .add_sized(
                            [PID_WIDTH, ROW_HEIGHT],
                            egui::Button::new("PID").frame(false),
                        )
                        .clicked()
                    {
                        state.sort = ProcessSort::PidAsc;
                    }
                    if ui
                        .add_sized(
                            [name_width, ROW_HEIGHT],
                            egui::Button::new("Nombre").frame(false),
                        )
                        .clicked()
                    {
                        state.sort = ProcessSort::NameAsc;
                    }
                    if ui
                        .add_sized(
                            [CPU_WIDTH, ROW_HEIGHT],
                            egui::Button::new("CPU").frame(false),
                        )
                        .clicked()
                    {
                        state.sort = ProcessSort::CpuDesc;
                    }
                    if ui
                        .add_sized(
                            [MEMORY_WIDTH, ROW_HEIGHT],
                            egui::Button::new("Memoria").frame(false),
                        )
                        .clicked()
                    {
                        state.sort = ProcessSort::MemoryDesc;
                    }
                    ui.add_sized([STATE_WIDTH, ROW_HEIGHT], egui::Label::new("Estado"));
                    ui.add_sized([USER_WIDTH, ROW_HEIGHT], egui::Label::new("Usuario"));
                    ui.end_row();

                    for process in &filtered[start..end] {
                        let selected = state.selected_pid == Some(process.pid);
                        if ui
                            .add_sized(
                                [PID_WIDTH, ROW_HEIGHT],
                                egui::SelectableLabel::new(selected, process.pid.to_string()),
                            )
                            .clicked()
                        {
                            state.selected_pid = Some(process.pid);
                        }
                        ui.add_sized(
                            [name_width, ROW_HEIGHT],
                            egui::Label::new(&process.name).truncate(),
                        );
                        ui.add_sized(
                            [CPU_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format!("{:.1}%", process.cpu_usage)),
                        );
                        ui.add_sized(
                            [MEMORY_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format!(
                                "{:.1} MB",
                                process.memory_bytes as f64 / 1024.0 / 1024.0
                            )),
                        );
                        ui.add_sized(
                            [STATE_WIDTH, ROW_HEIGHT],
                            egui::Label::new(&process.state).truncate(),
                        );
                        ui.add_sized(
                            [USER_WIDTH, ROW_HEIGHT],
                            egui::Label::new(&process.user).truncate(),
                        );
                        ui.end_row();
                    }
                });
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

            if is_protected(process.pid) {
                ui.label("Acciones deshabilitadas para este proceso.");
            } else {
                ui.horizontal(|ui| {
                    if ui.button("Terminar (SIGTERM)").clicked() {
                        state.pending_action = Some(PendingProcessAction {
                            pid: process.pid,
                            name: process.name.clone(),
                            kind: ProcessActionKind::Terminate,
                        });
                        state.pending_action_open = true;
                    }
                    if ui.button("Forzar (SIGKILL)").clicked() {
                        state.pending_action = Some(PendingProcessAction {
                            pid: process.pid,
                            name: process.name.clone(),
                            kind: ProcessActionKind::Kill,
                        });
                        state.pending_action_open = true;
                    }
                });
            }
        } else {
            state.selected_pid = None;
        }

        if let Some(action) = state.pending_action.clone() {
            if !state.pending_action_open {
                state.pending_action = None;
            } else {
                let mut open = state.pending_action_open;
                egui::Window::new("Confirmar acción")
                    .open(&mut open)
                    .show(ui.ctx(), |ui| {
                        let signal = match action.kind {
                            ProcessActionKind::Terminate => "SIGTERM (Terminate)",
                            ProcessActionKind::Kill => "SIGKILL (Kill)",
                        };
                        ui.label(format!(
                            "Se enviará {} al proceso {} ({})?",
                            signal, action.pid, action.name
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("Confirmar").clicked() {
                                let label = match action.kind {
                                    ProcessActionKind::Terminate => "Terminar",
                                    ProcessActionKind::Kill => "Forzar",
                                };
                                let result = match action.kind {
                                    ProcessActionKind::Terminate => {
                                        platform::terminate_process(action.pid)
                                    }
                                    ProcessActionKind::Kill => platform::kill_process(action.pid),
                                };
                                state.action_status = Some(match result {
                                    Ok(()) => (
                                        format!("{} pid {} realizado", label, action.pid),
                                        false,
                                    ),
                                    Err(error) => (
                                        format!("{} pid {} falló: {}", label, action.pid, error),
                                        true,
                                    ),
                                });
                                state.pending_action = None;
                                state.pending_action_open = false;
                            }
                            if ui.button("Cancelar").clicked() {
                                state.pending_action = None;
                                state.pending_action_open = false;
                            }
                        });
                    });
                state.pending_action_open = open;
                if !open {
                    state.pending_action = None;
                }
            }
        }
    }
}

fn is_protected(pid: usize) -> bool {
    pid == 1 || pid == std::process::id() as usize
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
