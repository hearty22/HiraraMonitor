use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{AppConfig, PersistedConfig, Theme, View};
use crate::metrics::{Collector, CollectorHandle, HistoryBuffer, SystemSnapshot};
use crate::ui::{
    cpu_panel, disks_panel, memory_panel, network_panel, processes_panel, sensors_panel,
    system_panel, ProcessTableState,
};

pub struct HiraraApp {
    config: AppConfig,
    collector: CollectorHandle,
    applied_theme: Option<Theme>,
    latest: Option<Arc<SystemSnapshot>>,
    collector_error: Option<String>,
    selected_view: View,
    config_saved: PersistedConfig,
    config_dirty_at: Option<std::time::Instant>,
    process_table: ProcessTableState,
    cpu_history: HistoryBuffer,
    memory_history: HistoryBuffer,
    swap_history: HistoryBuffer,
    network_rx_history: HistoryBuffer,
    network_tx_history: HistoryBuffer,
}

impl HiraraApp {
    pub fn new(config: AppConfig) -> Self {
        let history_len = config.history_len;
        let selected_view = config.initial_view;
        let process_table = ProcessTableState {
            page_size: config.process_page_size,
            sort: config.process_sort,
            filter: String::new(),
            page: 0,
            selected_pid: None,
            selected_pids: std::collections::HashSet::new(),
            selection_anchor: None,
            pending_action: None,
            pending_action_open: false,
            action_status: None,
        };
        let collector = Collector::spawn(config.clone());
        let saved_config = PersistedConfig::from(&config);

        Self {
            config,
            collector,
            applied_theme: None,
            latest: None,
            collector_error: None,
            selected_view,
            config_saved: saved_config,
            config_dirty_at: None,
            process_table,
            cpu_history: HistoryBuffer::new(history_len),
            memory_history: HistoryBuffer::new(history_len),
            swap_history: HistoryBuffer::new(history_len),
            network_rx_history: HistoryBuffer::new(history_len),
            network_tx_history: HistoryBuffer::new(history_len),
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        crate::ui::theme::apply(ctx, self.config.theme == crate::config::Theme::Dark);
    }

    fn ingest_snapshot(&mut self, snapshot: &SystemSnapshot) {
        let x = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs_f64())
            .unwrap_or(0.0);

        self.cpu_history.push(x, snapshot.cpu_global as f64);
        self.memory_history
            .push(x, snapshot.memory_used_percent() as f64);

        let swap_percent = if snapshot.memory.swap_total_bytes == 0 {
            0.0
        } else {
            snapshot.memory.swap_used_bytes as f64 / snapshot.memory.swap_total_bytes as f64 * 100.0
        };
        self.swap_history.push(x, swap_percent);

        let rx: f64 = snapshot
            .networks
            .iter()
            .map(|network| network.received_bytes_per_sec)
            .sum();
        let tx: f64 = snapshot
            .networks
            .iter()
            .map(|network| network.transmitted_bytes_per_sec)
            .sum();
        self.network_rx_history.push(x, rx);
        self.network_tx_history.push(x, tx);
    }

    fn receive_metrics(&mut self) {
        while let Ok(snapshot) = self.collector.receiver.try_recv() {
            self.ingest_snapshot(&snapshot);
            self.latest = Some(snapshot);
        }

        if let Ok(guard) = self.collector.error.lock() {
            if let Some(error) = guard.as_ref() {
                self.collector_error = Some(error.clone());
            }
        }
    }

    fn reconcile_persisted_config(&mut self) {
        self.config.process_sort = self.process_table.sort;
        self.config.process_page_size = self.process_table.page_size;
        self.config.initial_view = self.selected_view;

        let current = PersistedConfig::from(&self.config);
        if current != self.config_saved {
            if self.config_dirty_at.is_none() {
                self.config_dirty_at = Some(std::time::Instant::now());
            }
        } else {
            self.config_dirty_at = None;
        }

        if let Some(started_at) = self.config_dirty_at {
            if started_at.elapsed() >= std::time::Duration::from_secs(1) {
                let _ = self.config.save();
                self.config_saved = current;
                self.config_dirty_at = None;
            }
        }
    }
}

impl eframe::App for HiraraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.applied_theme != Some(self.config.theme) {
            self.apply_theme(ctx);
            self.applied_theme = Some(self.config.theme);
        }
        self.receive_metrics();
        self.reconcile_persisted_config();
        ctx.input(|input| {
            if input.key_pressed(egui::Key::Num1) {
                self.selected_view = View::Cpu;
            }
            if input.key_pressed(egui::Key::Num2) {
                self.selected_view = View::Memory;
            }
            if input.key_pressed(egui::Key::Num3) {
                self.selected_view = View::Disks;
            }
            if input.key_pressed(egui::Key::Num4) {
                self.selected_view = View::Network;
            }
            if input.key_pressed(egui::Key::Num5) {
                self.selected_view = View::Processes;
            }
            if input.key_pressed(egui::Key::Num6) {
                self.selected_view = View::Sensors;
            }
            if input.key_pressed(egui::Key::Num7) {
                self.selected_view = View::System;
            }
            if input.key_pressed(egui::Key::T) {
                self.config.theme = match self.config.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                };
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(
            self.config.refresh_interval.as_millis().clamp(16, 250) as u64,
        ));

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("HiraraMonitor");
                if ui.button("Tema").clicked() {
                    self.config.theme = match self.config.theme {
                        Theme::Light => Theme::Dark,
                        Theme::Dark => Theme::Light,
                    };
                }
                if ui
                    .button(if self.config.visual_alerts {
                        "Alertas: ON"
                    } else {
                        "Alertas: OFF"
                    })
                    .clicked()
                {
                    self.config.visual_alerts = !self.config.visual_alerts;
                }
                if let Some(snapshot) = self.latest.as_ref() {
                    ui.label(format!("CPU: {:.1}%", snapshot.cpu_global));
                    ui.label(format!("RAM: {:.1}%", snapshot.memory_used_percent()));
                }
            });
        });

        egui::SidePanel::left("navigation").show(ctx, |ui| {
            ui.heading("Vistas");
            for view in [
                View::Cpu,
                View::Memory,
                View::Disks,
                View::Network,
                View::Processes,
                View::Sensors,
                View::System,
            ] {
                let label = match view {
                    View::Cpu => "CPU",
                    View::Memory => "Memoria",
                    View::Disks => "Discos",
                    View::Network => "Red",
                    View::Processes => "Procesos",
                    View::Sensors => "Sensores",
                    View::System => "Sistema",
                };
                if ui
                    .selectable_label(self.selected_view == view, label)
                    .clicked()
                {
                    self.selected_view = view;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_view {
            View::Cpu => {
                cpu_panel::show(ui, self.latest.as_deref(), &self.cpu_history, &self.config)
            }
            View::Memory => memory_panel::show(
                ui,
                self.latest.as_deref(),
                &self.memory_history,
                &self.swap_history,
                &self.config,
            ),
            View::Disks => disks_panel::show(ui, self.latest.as_deref()),
            View::Network => network_panel::show(
                ui,
                self.latest.as_deref(),
                &self.network_rx_history,
                &self.network_tx_history,
            ),
            View::Processes => processes_panel::show(
                ui,
                self.latest.as_deref(),
                &mut self.process_table,
                &self.config,
            ),
            View::Sensors => sensors_panel::show(ui, self.latest.as_deref()),
            View::System => system_panel::show(ui, self.latest.as_deref()),
        });

        if let Some(error) = &self.collector_error {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, format!("Collector error: {}", error));
            });
        }

        if ctx.input(|input| input.viewport().close_requested()) {
            let _ = self.config.save();
        }
    }
}
