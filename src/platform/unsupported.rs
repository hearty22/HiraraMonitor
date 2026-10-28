use std::collections::HashMap;

use super::DiskIo;
use crate::platform::ProcessActions;

pub struct UnsupportedProcessActions;

impl ProcessActions for UnsupportedProcessActions {
    fn terminate(_pid: usize) -> Result<(), String> {
        Err("acciones de procesos no soportadas en esta plataforma".to_string())
    }

    fn kill(_pid: usize) -> Result<(), String> {
        Err("acciones de procesos no soportadas en esta plataforma".to_string())
    }
}

pub fn terminate_process(_pid: usize) -> Result<(), String> {
    <UnsupportedProcessActions as ProcessActions>::terminate(_pid)
}

pub fn disk_io() -> HashMap<String, DiskIo> {
    HashMap::new()
}

pub fn kill_process(_pid: usize) -> Result<(), String> {
    <UnsupportedProcessActions as ProcessActions>::kill(_pid)
}
