# Changelog

## Version 0.3.0 - 2026-10-27

### Added
- TOML preference persistence with `AppConfig`, `PersistedConfig` and `ConfigStore`.
- Safe process terminations and kills, guarded against protected PIDs.
- Multi-column process sort with ascending/descending indicators.
- Incremental process filtering and floating process details.
- Multi-selection and batch process actions.
- Visual alert configuration and color-coded CPU/RAM thresholds.
- Structured logging through `env_logger`.
- CI workflows for pull requests and automatic releases to `main`.
- Rust/egui collector backoff and stronger error reporting.

### Changed
- Process refresh moved to targeted `sysinfo` calls.
- Repaint interval capped to 250 ms and theme application cached.
- Stable fixed-column layouts continued across process, disk and sensor views.

### Fixed
- Collector failures no longer terminate the background thread silently.
- Batch action reporting now summarizes protected and failed PIDs.

## Version 0.2.0 - 2026-10-27

### Added
- Initial HiraraMonitor desktop application with egui.
- CPU view with global and per-core usage, frequencies and temperatures.
- Memory and swap monitoring with historical charts.
- Disk, network, process, sensor and system views.
- Process filtering, sorting, pagination and detail view.
- Light and dark themes.
- CLI options for refresh interval, history length and initial theme.
- Real-time rolling graphs with wall-clock timestamps.

### Changed
- Moved process pagination controls above the process table.
- Improved disks, sensors and system view layouts with fixed-width columns.
- Converted network interface summary to horizontal cards.
- Documented architecture and basic project structure.

### Fixed
- Graphs no longer rely on accumulating global X values.
- Process table no longer shifts because of process name length.
- Merged unrelated `main` and `develop` histories during integration.
