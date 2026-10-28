use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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
        let (tx, rx) = mpsc::sync_channel::<Arc<SystemSnapshot>>(1);
        let error = Arc::new(Mutex::new(None));
        let error_thread = Arc::clone(&error);

        let handle = thread::spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| run_collector(config, tx)));
            if result.is_err() {
                log::error!("collector panicked while refreshing metrics");
                if let Ok(mut guard) = error_thread.lock() {
                    *guard = Some("collector panicked".to_string());
                }
            }
        });

        // Prevent a detached panic from remaining unobserved by the main thread.
        drop(handle);

        CollectorHandle {
            receiver: rx,
            error,
        }
    }
}

fn set_error(error: &Arc<Mutex<Option<String>>>, value: Option<String>) {
    if let Ok(mut guard) = error.lock() {
        *guard = value;
    }
}

fn run_collector(
    config: AppConfig,
    tx: mpsc::SyncSender<Arc<SystemSnapshot>>,
) -> anyhow::Result<()> {
    let mut state = CollectorState::new();
    let mut attempt = 0usize;

    loop {
        let cycle_start = Instant::now();
        match collect_once(&mut state, &config) {
            Ok(snapshot) => {
                set_error(&state.error, None);
                if tx.send(snapshot).is_err() {
                    log::info!("metrics receiver closed; collector stopping");
                    return Ok(());
                }

                attempt = 0;
                let elapsed = cycle_start.elapsed();
                if elapsed > config.refresh_interval {
                    log::warn!(
                        "collector cycle took {:?}, longer than refresh interval {:?}",
                        elapsed,
                        config.refresh_interval
                    );
                }
                let remaining = config.refresh_interval.saturating_sub(elapsed);
                thread::sleep(remaining);
            }
            Err(error) => {
                log::error!("collector failed: {error:#}");
                set_error(&state.error, Some(error.to_string()));
                attempt = attempt.saturating_add(1);
                let backoff = match attempt {
                    1 => Duration::from_secs(1),
                    2 => Duration::from_secs(2),
                    3 => Duration::from_secs(5),
                    _ => Duration::from_secs(10),
                };
                log::warn!("backing off for {backoff:?} before retrying collector");
                thread::sleep(backoff);
            }
        }
    }
}

struct CollectorState {
    error: Arc<Mutex<Option<String>>>,
    system: System,
    disks: Disks,
    networks: Networks,
    components: Components,
    users: Users,
    last_network_time: Instant,
    last_network_bytes: std::collections::HashMap<String, (u64, u64)>,
    last_disk_time: Instant,
    last_disk_io: std::collections::HashMap<String, platform::DiskIo>,
}

impl CollectorState {
    fn new() -> Self {
        Self {
            error: Arc::new(Mutex::new(None)),
            system: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            users: Users::new_with_refreshed_list(),
            last_network_time: Instant::now(),
            last_network_bytes: std::collections::HashMap::new(),
            last_disk_time: Instant::now(),
            last_disk_io: std::collections::HashMap::new(),
        }
    }
}

fn collect_once(
    state: &mut CollectorState,
    _config: &AppConfig,
) -> anyhow::Result<Arc<SystemSnapshot>> {
    let now = Instant::now();

    // Compact, targeted refreshes are enough for the UI. `refresh_all()` is the expensive fallback.
    state.system.refresh_cpu_usage();
    state.system.refresh_memory();
    state
        .system
        .refresh_processes(sysinfo::ProcessesToUpdate::All);
    state.disks.refresh_list();
    for disk in state.disks.list_mut() {
        let _ = disk.refresh();
    }
    state.networks.refresh_list();
    state.networks.refresh();
    state.components.refresh_list();
    state.components.refresh();
    state.users.refresh_list();

    let cpu_temperature_celsius = state
        .components
        .list()
        .iter()
        .find(|component| {
            let label = component.label().to_lowercase();
            label.contains("cpu") || label.contains("package") || label.contains("core")
        })
        .map(|component| component.temperature());

    let per_core_temperatures: Vec<Option<f32>> = state
        .system
        .cpus()
        .iter()
        .enumerate()
        .map(|(index, _)| {
            state
                .components
                .list()
                .iter()
                .find(|component| {
                    let label = component.label().to_lowercase();
                    label.contains(&format!("core {}", index)) || label == format!("core {}", index)
                })
                .map(|component| component.temperature())
        })
        .collect();

    let memory = MemorySnapshot {
        total_bytes: state.system.total_memory(),
        used_bytes: state.system.used_memory(),
        free_bytes: state.system.free_memory(),
        available_bytes: state.system.available_memory(),
        swap_total_bytes: state.system.total_swap(),
        swap_used_bytes: state.system.used_swap(),
        cached_bytes: 0,
        buffers_bytes: 0,
    };

    let cpu_global = state.system.global_cpu_usage();
    let cpu_cores: Vec<CpuCoreSnapshot> = state
        .system
        .cpus()
        .iter()
        .enumerate()
        .map(|(index, cpu)| CpuCoreSnapshot {
            usage: cpu.cpu_usage(),
            frequency_mhz: cpu.frequency(),
            temperature_celsius: per_core_temperatures
                .get(index)
                .copied()
                .flatten()
                .or(cpu_temperature_celsius),
        })
        .collect();

    let current_disk_io = platform::disk_io();
    let disk_elapsed = now
        .duration_since(state.last_disk_time)
        .as_secs_f64()
        .max(0.001);
    let disks: Vec<DiskSnapshot> = state
        .disks
        .list()
        .iter()
        .map(|disk| {
            let name = disk.name().to_string_lossy().to_string();
            let io = current_disk_io.get(&name).cloned().unwrap_or_default();
            let (read_rate, write_rate) = if let Some(previous) = state.last_disk_io.get(&name) {
                (
                    io.read_bytes_total
                        .saturating_sub(previous.read_bytes_total) as f64
                        / disk_elapsed,
                    io.write_bytes_total
                        .saturating_sub(previous.write_bytes_total) as f64
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

    let network_elapsed = now
        .duration_since(state.last_network_time)
        .as_secs_f64()
        .max(0.001);
    let mut networks: Vec<NetworkSnapshot> = Vec::new();
    for (name, network) in state.networks.list() {
        let received_now = network.received();
        let transmitted_now = network.transmitted();
        let (prev_rx, prev_tx) = state
            .last_network_bytes
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

    let components: Vec<ComponentSnapshot> = state
        .components
        .list()
        .iter()
        .map(|component| ComponentSnapshot {
            label: component.label().to_string(),
            temperature_celsius: component.temperature(),
            max_temperature_celsius: component.max(),
        })
        .collect();

    let mut processes: Vec<ProcessSnapshot> = state
        .system
        .processes()
        .values()
        .map(|process| {
            let name = process.name().to_string_lossy().to_string();
            let user = process
                .user_id()
                .and_then(|uid| state.users.get_user_by_id(uid))
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
                command: process
                    .cmd()
                    .iter()
                    .map(|argument| argument.to_string_lossy().to_string())
                    .collect(),
                executable_path: process
                    .exe()
                    .map(|path| path.to_string_lossy().to_string())
                    .unwrap_or_default(),
            }
        })
        .collect();

    processes.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let load_average = System::load_average();
    let system_metadata = SystemMetadata {
        hostname: System::host_name().unwrap_or_default(),
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        kernel_version: System::kernel_version().unwrap_or_default(),
        architecture: std::env::consts::ARCH.to_string(),
        cpu_count: state.system.cpus().len(),
        uptime_seconds: System::uptime(),
        load_average_one: load_average.one,
        load_average_five: load_average.five,
        load_average_fifteen: load_average.fifteen,
    };

    let snapshot = SystemSnapshot {
        captured_at: now,
        cpu_global,
        cpu_temperature_celsius,
        cpu_cores,
        memory,
        disks,
        networks,
        processes,
        components,
        system: system_metadata,
    };

    state.last_network_time = now;
    state.last_network_bytes = state
        .networks
        .list()
        .iter()
        .map(|(name, network)| (name.clone(), (network.received(), network.transmitted())))
        .collect();
    state.last_disk_time = now;
    state.last_disk_io = current_disk_io;

    Ok(Arc::new(snapshot))
}
