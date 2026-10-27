#![allow(dead_code)]

use std::time::Instant;

#[derive(Clone, Debug, Default)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub cached_bytes: u64,
    pub buffers_bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct DiskSnapshot {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
}

#[derive(Clone, Debug, Default)]
pub struct NetworkSnapshot {
    pub name: String,
    pub received_bytes_per_sec: f64,
    pub transmitted_bytes_per_sec: f64,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessSnapshot {
    pub pid: usize,
    pub parent_pid: Option<usize>,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub state: String,
    pub user: String,
}

#[derive(Clone, Debug, Default)]
pub struct ComponentSnapshot {
    pub label: String,
    pub temperature_celsius: f32,
    pub max_temperature_celsius: f32,
}

#[derive(Clone, Debug, Default)]
pub struct SystemMetadata {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub architecture: String,
    pub cpu_count: usize,
    pub uptime_seconds: u64,
    pub load_average_one: f64,
    pub load_average_five: f64,
    pub load_average_fifteen: f64,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SystemSnapshot {
    pub captured_at: Instant,
    pub cpu_global: f32,
    pub cpu_cores: Vec<f32>,
    pub cpu_frequencies_mhz: Vec<u64>,
    pub memory: MemorySnapshot,
    pub disks: Vec<DiskSnapshot>,
    pub networks: Vec<NetworkSnapshot>,
    pub processes: Vec<ProcessSnapshot>,
    pub components: Vec<ComponentSnapshot>,
    pub system: SystemMetadata,
}

impl SystemSnapshot {
    pub fn memory_used_percent(&self) -> f32 {
        if self.memory.total_bytes == 0 {
            0.0
        } else {
            self.memory.used_bytes as f32 / self.memory.total_bytes as f32 * 100.0
        }
    }

    pub fn disk_used_percent(disk: &DiskSnapshot) -> f32 {
        if disk.total_bytes == 0 {
            0.0
        } else {
            (disk.total_bytes.saturating_sub(disk.available_bytes)) as f32 / disk.total_bytes as f32
                * 100.0
        }
    }
}
