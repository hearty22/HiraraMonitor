#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(not(target_os = "linux"))]
pub mod unsupported;

#[cfg(target_os = "linux")]
pub use linux::disk_io;

#[cfg(not(target_os = "linux"))]
pub use unsupported::disk_io;

#[derive(Clone, Debug, Default)]
pub struct DiskIo {
    pub read_bytes_total: u64,
    pub write_bytes_total: u64,
}

pub trait ProcessActions {
    fn terminate(pid: usize) -> Result<(), String>;

    fn kill(pid: usize) -> Result<(), String>;
}

pub fn terminate_process(pid: usize) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        linux::terminate_process(pid)
    }

    #[cfg(not(target_os = "linux"))]
    {
        unsupported::terminate_process(pid)
    }
}

pub fn kill_process(pid: usize) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        linux::kill_process(pid)
    }

    #[cfg(not(target_os = "linux"))]
    {
        unsupported::kill_process(pid)
    }
}
