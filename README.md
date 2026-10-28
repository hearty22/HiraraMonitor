# HiraraMonitor

HiraraMonitor is a desktop system monitor built with Rust and egui, inspired by btop. It ships a native UI with real-time charts and a collector thread that keeps metric refresh outside the render loop.

## Features

- CPU monitoring: global and per-core usage, frequency, temperature when available.
- Memory and swap monitoring with rolling charts.
- Disk monitoring with usage, filesystem, mount point and read/write throughput.
- Network monitoring with RX/TX rates and totals.
- Process monitoring with filter, sorting, pagination, selection, details, safe terminate and kill.
- Sensors and system metadata views.
- Light and dark themes with a dedicated theme application.
- Visual alerts for CPU, RAM and process values with configurable thresholds.
- Preferences persisted to a TOML file, with CLI precedence.
- Linux-specific disk throughput from `/proc/diskstats`.
- Fixed-width layouts for stable process, disk and sensor tables.

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release
```

Optional arguments:

```bash
cargo run --release -- --refresh-ms 500 --history 300 --light
cargo run --release -- --config ./hirara.toml
```

Available options:

- `--refresh-ms <milliseconds>`: metric refresh interval; minimum 250ms.
- `--history <samples>`: retained sample count; clamped to 30..5000.
- `--light`: start in light theme.
- `--dark`: start in dark theme.
- `--config <path>`: config file path. Environment override: `HIRARA_CONFIG`.

Configuration paths:

- default on Linux: `~/.config/hiraramonitor/config.toml`
- explicit path always takes priority.

## Architecture

The app separates collection, snapshot publishing, UI state and rendering.

- `collector.rs` refreshes system information on a background thread.
- `snapshot.rs` defines the immutable data model.
- `history.rs` stores circular rolling buffers for charts.
- `app.rs` ingests snapshots, owns routing and persisted UI preferences.
- `ui/*` renders each view.
- `platform/*` contains Linux and unsupported implementations.
- Settings persist to the file chosen by `--config`/`HIRARA_CONFIG`, or the default path on Linux.

See `docs/ARCHITECTURE.md` for details.

## Keyboard shortcuts

- `1`–`7`: switch views
- `T`: toggle theme
- Top bar button: toggle visual alerts

## Controls

- Process selection: click the PID cell.
- Process floating detail: opens for the selected PID.
- Sort columns: click a header to toggle direction.
- Multi-select: Ctrl/Shift click process rows.
- Batch actions: "Terminar seleccionados" and "Forzar seleccion" require confirmation.

## Project layout

```text
src/
├── app.rs
├── config.rs
├── config_store.rs
├── main.rs
├── metrics/
│   ├── collector.rs
│   ├── history.rs
│   └── snapshot.rs
├── platform/
│   ├── linux.rs
│   ├── mod.rs
│   └── unsupported.rs
├── ui/
│   ├── components.rs
│   ├── cpu_panel.rs
│   ├── disks_panel.rs
│   ├── memory_panel.rs
│   ├── network_panel.rs
│   ├── processes_panel.rs
│   ├── sensors_panel.rs
│   ├── system_panel.rs
│   └── theme.rs
└── util/
    └── format.rs
```

## Notes

- Linux is the supported target; unsupported targets provide fallback disk metrics.
- GPU monitoring is intentionally not implemented.
- Process kill/terminate is implemented with PID 1 and own-process protection.
- The UI is intentionally read-only except for guarded process signals.
- Release builds and CI are available for Linux.

## License

MIT
