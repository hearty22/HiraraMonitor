# Changelog

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
```