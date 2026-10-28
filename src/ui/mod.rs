use crate::metrics::ProcessSnapshot;

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
pub enum ProcessActionKind {
    Terminate,
    Kill,
}

#[derive(Clone, Debug)]
pub struct PendingProcessAction {
    pub pids: Vec<usize>,
    pub name: Option<String>,
    pub kind: ProcessActionKind,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProcessSort {
    PidAsc,
    PidDesc,
    NameAsc,
    NameDesc,
    #[default]
    CpuDesc,
    CpuAsc,
    MemoryDesc,
    MemoryAsc,
    StateAsc,
    StateDesc,
    UserAsc,
    UserDesc,
}

impl ProcessSort {
    pub fn label(self) -> &'static str {
        match self {
            Self::PidAsc => "PID asc",
            Self::PidDesc => "PID desc",
            Self::NameAsc => "Nombre asc",
            Self::NameDesc => "Nombre desc",
            Self::CpuAsc => "CPU asc",
            Self::CpuDesc => "CPU desc",
            Self::MemoryAsc => "Memoria asc",
            Self::MemoryDesc => "Memoria desc",
            Self::StateAsc => "Estado asc",
            Self::StateDesc => "Estado desc",
            Self::UserAsc => "Usuario asc",
            Self::UserDesc => "Usuario desc",
        }
    }

    pub fn toggled(self, column: &str) -> Self {
        match (column, self) {
            ("pid", Self::PidAsc) => Self::PidDesc,
            ("pid", _) => Self::PidAsc,
            ("name", Self::NameAsc) => Self::NameDesc,
            ("name", _) => Self::NameAsc,
            ("cpu", Self::CpuAsc) => Self::CpuDesc,
            ("cpu", _) => Self::CpuAsc,
            ("memory", Self::MemoryAsc) => Self::MemoryDesc,
            ("memory", _) => Self::MemoryAsc,
            ("state", Self::StateAsc) => Self::StateDesc,
            ("state", _) => Self::StateAsc,
            ("user", Self::UserAsc) => Self::UserDesc,
            ("user", _) => Self::UserAsc,
            _ => self,
        }
    }

    pub fn indicator(self, column: &str) -> &'static str {
        match (column, self) {
            ("pid", Self::PidAsc) => " ↑",
            ("pid", Self::PidDesc) => " ↓",
            ("name", Self::NameAsc) => " ↑",
            ("name", Self::NameDesc) => " ↓",
            ("cpu", Self::CpuAsc) => " ↑",
            ("cpu", Self::CpuDesc) => " ↓",
            ("memory", Self::MemoryAsc) => " ↑",
            ("memory", Self::MemoryDesc) => " ↓",
            ("state", Self::StateAsc) => " ↑",
            ("state", Self::StateDesc) => " ↓",
            ("user", Self::UserAsc) => " ↑",
            ("user", Self::UserDesc) => " ↓",
            _ => "",
        }
    }

    #[allow(dead_code)]
    pub fn is_column(self, column: &str) -> bool {
        matches!(
            (column, self),
            ("pid", Self::PidAsc | Self::PidDesc)
                | ("name", Self::NameAsc | Self::NameDesc)
                | ("cpu", Self::CpuAsc | Self::CpuDesc)
                | ("memory", Self::MemoryAsc | Self::MemoryDesc)
                | ("state", Self::StateAsc | Self::StateDesc)
                | ("user", Self::UserAsc | Self::UserDesc)
        )
    }

    pub fn compare(self, left: &ProcessSnapshot, right: &ProcessSnapshot) -> std::cmp::Ordering {
        let ascending = matches!(
            self,
            Self::PidAsc
                | Self::NameAsc
                | Self::CpuAsc
                | Self::MemoryAsc
                | Self::StateAsc
                | Self::UserAsc
        );
        let order = match self {
            Self::PidAsc | Self::PidDesc => left.pid.cmp(&right.pid),
            Self::NameAsc | Self::NameDesc => left.name.cmp(&right.name),
            Self::CpuAsc | Self::CpuDesc => left
                .cpu_usage
                .partial_cmp(&right.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal),
            Self::MemoryAsc | Self::MemoryDesc => left.memory_bytes.cmp(&right.memory_bytes),
            Self::StateAsc | Self::StateDesc => left.state.cmp(&right.state),
            Self::UserAsc | Self::UserDesc => left.user.cmp(&right.user),
        };
        if ascending {
            order
        } else {
            order.reverse()
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
    pub selected_pids: std::collections::HashSet<usize>,
    pub selection_anchor: Option<usize>,
    pub pending_action: Option<PendingProcessAction>,
    pub pending_action_open: bool,
    pub action_status: Option<(String, bool)>,
}

impl Default for ProcessTableState {
    fn default() -> Self {
        Self {
            filter: String::new(),
            sort: ProcessSort::CpuDesc,
            page: 0,
            page_size: 25,
            selected_pid: None,
            selected_pids: std::collections::HashSet::new(),
            selection_anchor: None,
            pending_action: None,
            pending_action_open: false,
            action_status: None,
        }
    }
}
