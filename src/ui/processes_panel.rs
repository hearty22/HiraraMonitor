use egui::Ui;

use crate::metrics::{ProcessSnapshot, SystemSnapshot};
use crate::platform;
use crate::ui::{PendingProcessAction, ProcessActionKind, ProcessSort, ProcessTableState};

const PID_WIDTH: f32 = 70.0;
const CPU_WIDTH: f32 = 80.0;
const MEMORY_WIDTH: f32 = 110.0;
const STATE_WIDTH: f32 = 95.0;
const USER_WIDTH: f32 = 120.0;
const ROW_HEIGHT: f32 = 18.0;

pub fn show(
    ui: &mut Ui,
    snapshot: Option<&SystemSnapshot>,
    state: &mut ProcessTableState,
    config: &crate::config::AppConfig,
) {
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

    filtered.sort_by(|a, b| state.sort.compare(a, b));

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
        ui.label(format!(
            "Total: {} | Seleccionados: {}",
            filtered.len(),
            state.selected_pids.len()
        ));
        if !state.selected_pids.is_empty() && ui.button("Limpiar selección").clicked() {
            state.selected_pids.clear();
            state.selected_pid = None;
            state.selection_anchor = None;
        }
        if !state.selected_pids.is_empty() && ui.button("Seleccionar visibles").clicked() {
            for process in &filtered {
                state.selected_pids.insert(process.pid);
            }
        }
    });

    if !state.selected_pids.is_empty() {
        ui.horizontal(|ui| {
            ui.label(format!("{} seleccionados", state.selected_pids.len()));
            if ui.button("Terminar seleccionados").clicked() {
                let pids = state.selected_pids.iter().copied().collect();
                state.pending_action = Some(PendingProcessAction {
                    pids,
                    name: None,
                    kind: ProcessActionKind::Terminate,
                });
                state.pending_action_open = true;
            }
            if ui.button("Forzar selección").clicked() {
                let pids = state.selected_pids.iter().copied().collect();
                state.pending_action = Some(PendingProcessAction {
                    pids,
                    name: None,
                    kind: ProcessActionKind::Kill,
                });
                state.pending_action_open = true;
            }
        });
    }

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
                            egui::Button::new(format!("PID{}", state.sort.indicator("pid")))
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.sort = state.sort.toggled("pid");
                    }
                    if ui
                        .add_sized(
                            [name_width, ROW_HEIGHT],
                            egui::Button::new(format!("Nombre{}", state.sort.indicator("name")))
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.sort = state.sort.toggled("name");
                    }
                    if ui
                        .add_sized(
                            [CPU_WIDTH, ROW_HEIGHT],
                            egui::Button::new(format!("CPU{}", state.sort.indicator("cpu")))
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.sort = state.sort.toggled("cpu");
                    }
                    if ui
                        .add_sized(
                            [MEMORY_WIDTH, ROW_HEIGHT],
                            egui::Button::new(format!("Memoria{}", state.sort.indicator("memory")))
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.sort = state.sort.toggled("memory");
                    }
                    if ui
                        .add_sized(
                            [STATE_WIDTH, ROW_HEIGHT],
                            egui::Button::new(format!("Estado{}", state.sort.indicator("state")))
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.sort = state.sort.toggled("state");
                    }
                    if ui
                        .add_sized(
                            [USER_WIDTH, ROW_HEIGHT],
                            egui::Button::new(format!("Usuario{}", state.sort.indicator("user")))
                                .frame(false),
                        )
                        .clicked()
                    {
                        state.sort = state.sort.toggled("user");
                    }
                    ui.end_row();

                    for (row_offset, process) in filtered[start..end].iter().enumerate() {
                        let row_index = start + row_offset;
                        let selected = state.selected_pid == Some(process.pid)
                            || state.selected_pids.contains(&process.pid);
                        if ui
                            .add_sized(
                                [PID_WIDTH, ROW_HEIGHT],
                                egui::SelectableLabel::new(selected, process.pid.to_string()),
                            )
                            .clicked()
                        {
                            let modifiers = ui.input(|input| input.modifiers);
                            if modifiers.command || modifiers.ctrl {
                                if !state.selected_pids.remove(&process.pid) {
                                    state.selected_pids.insert(process.pid);
                                }
                                state.selection_anchor = Some(row_index);
                            } else if modifiers.shift {
                                let start_index = state.selection_anchor.unwrap_or(row_index);
                                let (low, high) =
                                    (start_index.min(row_index), start_index.max(row_index));
                                for index in low..=high {
                                    if let Some(candidate) = filtered.get(index) {
                                        state.selected_pids.insert(candidate.pid);
                                    }
                                }
                            } else {
                                state.selected_pids.clear();
                                state.selected_pids.insert(process.pid);
                                state.selected_pid = Some(process.pid);
                                state.selection_anchor = Some(row_index);
                            }
                        }
                        ui.add_sized(
                            [name_width, ROW_HEIGHT],
                            egui::Label::new(&process.name).truncate(),
                        );
                        ui.add_sized(
                            [CPU_WIDTH, ROW_HEIGHT],
                            egui::Label::new(
                                egui::RichText::new(format!("{:.1}%", process.cpu_usage)).color(
                                    if config.visual_alerts {
                                        crate::ui::components::alert_color(
                                            process.cpu_usage,
                                            config.cpu_warn,
                                            config.cpu_crit,
                                            config.visual_alerts,
                                        )
                                    } else {
                                        crate::ui::theme::TEXT
                                    },
                                ),
                            ),
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

    let selected_process = state.selected_pid.and_then(|selected_pid| {
        snapshot
            .processes
            .iter()
            .find(|process| process.pid == selected_pid)
            .cloned()
    });

    if state.selected_pid.is_some() && selected_process.is_none() {
        state.selected_pid = None;
        state.pending_action = None;
        state.pending_action_open = false;
    }

    if let Some(process) = selected_process {
        let mut detail_open = true;
        egui::Window::new("Detalle del proceso")
            .default_width(420.0)
            .resizable(false)
            .collapsible(false)
            .open(&mut detail_open)
            .show(ui.ctx(), |ui| {
                render_process_detail(ui, &process, state, config);
            });
        if !detail_open {
            state.selected_pid = None;
            state.pending_action = None;
            state.pending_action_open = false;
        }
    }

    if let Some(action) = state.pending_action.clone() {
        if !state.pending_action_open {
            state.pending_action = None;
        } else {
            let mut open = state.pending_action_open;
            egui::Window::new("Confirmar accion")
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    let signal = match action.kind {
                        ProcessActionKind::Terminate => "SIGTERM (Terminate)",
                        ProcessActionKind::Kill => "SIGKILL (Kill)",
                    };
                    let target = if let Some(pid) = action.pids.first().copied() {
                        if action.pids.len() == 1 {
                            if let Some(name) = &action.name {
                                format!("el proceso {pid} ({name})")
                            } else {
                                format!("el proceso {pid}")
                            }
                        } else {
                            format!("{} procesos seleccionados", action.pids.len())
                        }
                    } else {
                        "ningun proceso".to_string()
                    };
                    ui.label(format!("Se enviara {} a {}?", signal, target));
                    ui.horizontal(|ui| {
                        if ui.button("Confirmar").clicked() {
                            let label = match action.kind {
                                ProcessActionKind::Terminate => "Terminar",
                                ProcessActionKind::Kill => "Forzar",
                            };
                            let mut successes = 0;
                            let mut failures = Vec::new();
                            for pid in &action.pids {
                                if is_protected(*pid) {
                                    failures.push(format!("pid {pid} protegido"));
                                    continue;
                                }
                                let result = match action.kind {
                                    ProcessActionKind::Terminate => {
                                        platform::terminate_process(*pid)
                                    }
                                    ProcessActionKind::Kill => platform::kill_process(*pid),
                                };
                                match result {
                                    Ok(_) => successes += 1,
                                    Err(error) => failures.push(format!("pid {pid}: {error}")),
                                }
                            }
                            let total = action.pids.len();
                            let error = !failures.is_empty();
                            let mut message = format!("{label}: {successes}/{total} exitosos");
                            if !failures.is_empty() {
                                failures.truncate(4);
                                message.push_str("; ");
                                message.push_str(&failures.join(", "));
                                if total > 4 {
                                    message.push_str(", ...");
                                }
                            }
                            state.action_status = Some((message, error));
                            state.selected_pids.clear();
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

fn render_process_detail(
    ui: &mut egui::Ui,
    process: &ProcessSnapshot,
    state: &mut ProcessTableState,
    config: &crate::config::AppConfig,
) {
    ui.heading("Detalle del proceso");
    if ui.button("Cerrar detalle").clicked() {
        state.selected_pid = None;
        state.pending_action = None;
        state.pending_action_open = false;
    }
    ui.separator();
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
            ui.label(
                egui::RichText::new(format!("{:.1}%", process.cpu_usage)).color(
                    crate::ui::components::alert_color(
                        process.cpu_usage,
                        config.cpu_warn,
                        config.cpu_crit,
                        config.visual_alerts,
                    ),
                ),
            );
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
                    pids: vec![process.pid],
                    name: Some(process.name.clone()),
                    kind: ProcessActionKind::Terminate,
                });
                state.pending_action_open = true;
            }
            if ui.button("Forzar (SIGKILL)").clicked() {
                state.pending_action = Some(PendingProcessAction {
                    pids: vec![process.pid],
                    name: Some(process.name.clone()),
                    kind: ProcessActionKind::Kill,
                });
                state.pending_action_open = true;
            }
        });
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
