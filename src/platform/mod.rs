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
