use std::sync::Arc;
use std::time::Instant;

use crate::config::{AppConfig, Theme, View};
use crate::metrics::{Collector, CollectorHandle, HistoryBuffer, SystemSnapshot};
use crate::ui::{
    cpu_panel, disks_panel, memory_panel, network_panel, processes_panel, sensors_panel,
    system_panel, ProcessTableState,
};

pub struct HiraraApp {
    config: AppConfig,
    collector: CollectorHandle,
    latest: Option<Arc<SystemSnapshot>>,
    collector_error: Option<String>,
    selected_view: View,
    process_table: ProcessTableState,
    cpu_history: HistoryBuffer,
    memory_history: HistoryBuffer,
    swap_history: HistoryBuffer,
    network_rx_history: HistoryBuffer,
    network_tx_history: HistoryBuffer,
    start_time: Instant,
}

impl HiraraApp {
    pub fn new() -> Self {
        let config = AppConfig::from_args();
        let history_len = config.history_len;
        let process_table = ProcessTableState {
            page_size: config.process_page_size,
            ..ProcessTableState::default()
        };
        let collector = Collector::spawn(config.clone());

        Self {
            config,
            collector,
            latest: None,
            collector_error: None,
            selected_view: View::Cpu,
            process_table,
            cpu_history: HistoryBuffer::new(history_len),
            memory_history: HistoryBuffer::new(history_len),
            swap_history: HistoryBuffer::new(history_len),
            network_rx_history: HistoryBuffer::new(history_len),
            network_tx_history: HistoryBuffer::new(history_len),
            start_time: Instant::now(),
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let mut visuals = match self.config.theme {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        };

        if self.config.theme == crate::config::Theme::Dark {
            visuals.panel_fill = egui::Color32::from_rgb(22, 24, 29);
            visuals.faint_bg_color = egui::Color32::from_rgb(30, 33, 40);
        } else {
            visuals.panel_fill = egui::Color32::from_rgb(244, 246, 250);
            visuals.faint_bg_color = egui::Color32::from_rgb(232, 236, 243);
        }

        ctx.set_visuals(visuals);
    }

    fn ingest_snapshot(&mut self, snapshot: &SystemSnapshot) {
        let x = snapshot
            .captured_at
            .duration_since(self.start_time)
            .as_secs_f64();

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
}

impl eframe::App for HiraraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        self.receive_metrics();

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

        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("HiraraMonitor");
                if ui.button("Tema").clicked() {
                    self.config.theme = match self.config.theme {
                        Theme::Light => Theme::Dark,
                        Theme::Dark => Theme::Light,
                    };
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
            View::Cpu => cpu_panel::show(ui, self.latest.as_deref(), &self.cpu_history),
            View::Memory => memory_panel::show(
                ui,
                self.latest.as_deref(),
                &self.memory_history,
                &self.swap_history,
            ),
            View::Disks => disks_panel::show(ui, self.latest.as_deref()),
            View::Network => network_panel::show(
                ui,
                self.latest.as_deref(),
                &self.network_rx_history,
                &self.network_tx_history,
            ),
            View::Processes => {
                processes_panel::show(ui, self.latest.as_deref(), &mut self.process_table)
            }
            View::Sensors => sensors_panel::show(ui, self.latest.as_deref()),
            View::System => system_panel::show(ui, self.latest.as_deref()),
        });

        if let Some(error) = &self.collector_error {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, format!("Collector error: {}", error));
            });
        }
    }
}
