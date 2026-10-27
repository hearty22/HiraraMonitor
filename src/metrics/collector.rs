use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

use sysinfo::{Components, Disks, Networks, System, Users};

use crate::config::AppConfig;
use crate::metrics::snapshot::*;
use crate::platform;

pub struct CollectorHandle {
    pub receiver: mpsc::Receiver<Arc<SystemSnapshot>>,
    pub error: Arc<Mutex<Option<String>>>,
}

pub struct Collector;

impl Collector {
    pub fn spawn(config: AppConfig) -> CollectorHandle {
        let (tx, rx) = mpsc::sync_channel::<Arc<SystemSnapshot>>(2);
        let error = Arc::new(Mutex::new(None));
        let error_thread = Arc::clone(&error);

        thread::spawn(move || {
            if let Err(err) = run_collector(config, tx) {
                if let Ok(mut guard) = error_thread.lock() {
                    *guard = Some(err.to_string());
                }
            }
        });

        CollectorHandle {
            receiver: rx,
            error,
        }
    }
}

fn run_collector(
    config: AppConfig,
    tx: mpsc::SyncSender<Arc<SystemSnapshot>>,
) -> anyhow::Result<()> {
    let mut system = System::new_all();
    let mut disks_sys = Disks::new_with_refreshed_list();
    let mut networks_sys = Networks::new_with_refreshed_list();
    let mut components_sys = Components::new_with_refreshed_list();
    let mut users_sys = Users::new_with_refreshed_list();

    let mut last_network_time = Instant::now();
    let mut last_network_bytes: HashMap<String, (u64, u64)> = HashMap::new();

    let mut last_disk_time = Instant::now();
    let mut last_disk_io = platform::disk_io();

    loop {
        let cycle_start = Instant::now();
        let now = Instant::now();

        // Refresh core system data.
        system.refresh_all();

        // Refresh extended collectors.
        disks_sys.refresh_list();
        for disk in disks_sys.list_mut() {
            let _ = disk.refresh();
        }
        networks_sys.refresh_list();
        networks_sys.refresh();
        components_sys.refresh_list();
        components_sys.refresh();
        users_sys.refresh_list();

        // Memory
        let memory = MemorySnapshot {
            total_bytes: system.total_memory(),
            used_bytes: system.used_memory(),
            available_bytes: system.available_memory(),
            swap_total_bytes: system.total_swap(),
            swap_used_bytes: system.used_swap(),
            cached_bytes: 0,
            buffers_bytes: 0,
        };

        // CPU
        let cpu_global = system.global_cpu_usage();
        let cpu_cores: Vec<f32> = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let cpu_frequencies_mhz: Vec<u64> =
            system.cpus().iter().map(|cpu| cpu.frequency()).collect();

        // Disks
        let current_disk_io = platform::disk_io();
        let disk_elapsed = now.duration_since(last_disk_time).as_secs_f64().max(0.001);
        let disks: Vec<DiskSnapshot> = disks_sys
            .list()
            .iter()
            .map(|disk| {
                let name = disk.name().to_string_lossy().to_string();
                let io = current_disk_io.get(&name).cloned().unwrap_or_default();
                let (read_rate, write_rate) = if let Some(previous) = last_disk_io.get(&name) {
                    (
                        io.read_bytes_total
                            .saturating_sub(previous.read_bytes_total)
                            as f64
                            / disk_elapsed,
                        io.write_bytes_total
                            .saturating_sub(previous.write_bytes_total)
                            as f64
                            / disk_elapsed,
                    )
                } else {
                    (0.0, 0.0)
                };

                DiskSnapshot {
                    name,
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    total_bytes: disk.total_space(),
                    available_bytes: disk.available_space(),
                    read_bytes_per_sec: read_rate,
                    write_bytes_per_sec: write_rate,
                }
            })
            .collect();

        // Networks
        let network_elapsed = now
            .duration_since(last_network_time)
            .as_secs_f64()
            .max(0.001);
        let mut networks: Vec<NetworkSnapshot> = Vec::new();
        for (name, network) in networks_sys.list() {
            let received_now = network.received();
            let transmitted_now = network.transmitted();
            let (prev_rx, prev_tx) = last_network_bytes
                .get(name)
                .copied()
                .unwrap_or((received_now, transmitted_now));
            let rx_delta = received_now.saturating_sub(prev_rx) as f64 / network_elapsed;
            let tx_delta = transmitted_now.saturating_sub(prev_tx) as f64 / network_elapsed;

            networks.push(NetworkSnapshot {
                name: name.clone(),
                received_bytes_per_sec: rx_delta,
                transmitted_bytes_per_sec: tx_delta,
                total_received_bytes: network.total_received(),
                total_transmitted_bytes: network.total_transmitted(),
            });
        }

        // Components
        let components: Vec<ComponentSnapshot> = components_sys
            .list()
            .iter()
            .map(|component| ComponentSnapshot {
                label: component.label().to_string(),
                temperature_celsius: component.temperature(),
                max_temperature_celsius: component.max(),
            })
            .collect();

        // Processes
        let mut processes: Vec<ProcessSnapshot> = system
            .processes()
            .values()
            .map(|process| {
                let name = process.name().to_string_lossy().to_string();
                let user = process
                    .user_id()
                    .and_then(|uid| users_sys.get_user_by_id(uid))
                    .map(|user| user.name().to_string())
                    .unwrap_or_default();
                ProcessSnapshot {
                    pid: process.pid().as_u32() as usize,
                    parent_pid: process.parent().map(|pid| pid.as_u32() as usize),
                    name,
                    cpu_usage: process.cpu_usage(),
                    memory_bytes: process.memory(),
                    state: format!("{:?}", process.status()),
                    user,
                }
            })
            .collect();

        processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // System metadata
        let load_average = System::load_average();
        let system_metadata = SystemMetadata {
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_count: system.cpus().len(),
            uptime_seconds: System::uptime(),
            load_average_one: load_average.one,
            load_average_five: load_average.five,
            load_average_fifteen: load_average.fifteen,
        };

        let snapshot = SystemSnapshot {
            captured_at: now,
            cpu_global,
            cpu_cores,
            cpu_frequencies_mhz,
            memory,
            disks,
            networks,
            processes,
            components,
            system: system_metadata,
        };

        let _ = tx.try_send(Arc::new(snapshot));

        // Save previous samples for deltas.
        last_network_time = now;
        last_network_bytes = networks_sys
            .list()
            .iter()
            .map(|(name, network)| (name.clone(), (network.received(), network.transmitted())))
            .collect();
        last_disk_time = now;
        last_disk_io = current_disk_io;

        let elapsed = cycle_start.elapsed();
        if elapsed < config.refresh_interval {
            thread::sleep(config.refresh_interval - elapsed);
        }
    }
}
