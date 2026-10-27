#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

#[allow(dead_code)]
pub fn format_bytes_per_second(bytes_per_second: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_second as u64))
}

#[allow(dead_code)]
pub fn format_percent(value: f32) -> String {
    format!("{:.1}%", value)
}

#[allow(dead_code)]
pub fn format_epoch_time(seconds: f64) -> String {
    if let Some(datetime) =
        chrono::DateTime::from_timestamp(seconds as i64, (seconds.fract() * 1_000_000_000.0) as u32)
    {
        datetime
            .with_timezone(&chrono::Local)
            .format("%H:%M:%S")
            .to_string()
    } else {
        format!("{:.0}", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, format_percent};

    #[test]
    fn formats_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
    }

    #[test]
    fn formats_percent() {
        assert_eq!(format_percent(12.34), "12.3%");
    }
}
