use std::collections::HashMap;
use std::fs;

use super::DiskIo;

pub fn disk_io() -> HashMap<String, DiskIo> {
    let mut result = HashMap::new();
    let Ok(content) = fs::read_to_string("/proc/diskstats") else {
        return result;
    };

    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // /proc/diskstats: major minor name reads_completed ... sectors_read ... writes ... sectors_written
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
