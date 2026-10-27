pub mod components;
pub mod cpu_panel;
pub mod disks_panel;
pub mod layout;
pub mod memory_panel;
pub mod network_panel;
pub mod processes_panel;
pub mod sensors_panel;
pub mod system_panel;
pub mod theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessSort {
    CpuDesc,
    MemoryDesc,
    NameAsc,
    PidAsc,
}

impl ProcessSort {
    pub fn label(self) -> &'static str {
        match self {
            Self::CpuDesc => "CPU desc",
            Self::MemoryDesc => "Memoria desc",
            Self::NameAsc => "Nombre asc",
            Self::PidAsc => "PID asc",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcessTableState {
    pub filter: String,
    pub sort: ProcessSort,
    pub page: usize,
    pub page_size: usize,
    pub selected_pid: Option<usize>,
}

impl Default for ProcessTableState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            sort: ProcessSort::CpuDesc,
            page: 0,
            page_size: 25,
            selected_pid: None,
        }
    }
}
