use std::collections::HashMap;

use crate::platform::ProcessActions;
use super::DiskIo;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

#[derive(Clone, Copy)]
pub struct LinuxProcessActions;

impl ProcessActions for LinuxProcessActions {
    fn terminate(pid: usize) -> Result<(), String> {
        send(pid, Signal::Term)
    }

    fn kill(pid: usize) -> Result<(), String> {
        send(pid, Signal::Kill)
    }
}

pub fn terminate_process(pid: usize) -> Result<(), String> {
    <LinuxProcessActions as ProcessActions>::terminate(pid)
}

pub fn disk_io() -> HashMap<String, DiskIo> {
        use std::fs;

    let mut result = HashMap::new();
    let Ok(content) = fs::read_to_string("/proc/diskstats") else {
        return result;
    };

    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 14 {
            continue;
        }
        let name = fields[2].to_string();
        let sectors_read = fields[5].parse::<u64>().unwrap_or(0);
        let sectors_written = fields[9].parse::<u64>().unwrap_or(0);
        result.insert(
            name,
            DiskIo {
                read_bytes_total: sectors_read.saturating_mul(512),
                write_bytes_total: sectors_written.saturating_mul(512),
            },
        );
    }

    result
}

pub fn kill_process(pid: usize) -> Result<(), String> {
    <LinuxProcessActions as ProcessActions>::kill(pid)
}


fn send(pid: usize, signal: Signal) -> Result<(), String> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All);

    match system.process(Pid::from(pid)) {
        Some(process) => match process.kill_with(signal) {
            Some(true) => Ok(()),
            Some(false) => Err(format!("el proceso {pid} no pudo recibir la señal")),
            None => Err(format!("señal no soportada para el proceso {pid}")),
        },
        None => Err(format!("el proceso {pid} ya no está en ejecución")),
    }
}
