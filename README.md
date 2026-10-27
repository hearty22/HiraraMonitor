# HiraraMonitor

HiraraMonitor is a desktop system monitor built with Rust and egui.  
It is inspired by tools such as btop, with a focus on native performance, low overhead and a clear multi-view interface.

## Features

- CPU monitoring, including global usage, per-core usage, frequency and temperature when available.
- Memory and swap monitoring with historical charts.
- Network interface monitoring with RX/TX rates and totals.
- Disk monitoring with usage, filesystem, mount point and read/write throughput.
- Process monitoring with filtering, sorting, pagination and selected process details.
- Temperature sensors and system metadata.
- Separate views for CPU, memory, disks, network, processes, sensors and system.
- Light and dark themes.
- Real-time rolling graphs with wall-clock time labels.
- No persistent global configuration is written automatically.

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
```

Available options:

- `--refresh-ms <milliseconds>`: metric refresh interval. Minimum is 250ms.
- `--history <samples>`: number of samples retained. Clamped to 30..5000.
- `--light`: start in light theme.
- `--dark`: start in dark theme.

## Architecture

HiraraMonitor separates metric collection from rendering.

- A collector thread refreshes system information through `sysinfo`.
- Snapshots are sent to the UI through a bounded channel.
- The UI stores the latest snapshot and updates circular history buffers.
- Panels render the current view from the latest snapshot.
- Linux-specific disk throughput is read from `/proc/diskstats`.

See `docs/ARCHITECTURE.md` for details.

## Project layout

```text
src/
├── app.rs
├── config.rs
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
│   ├── app panels...
│   ├── cpu_panel.rs
│   ├── disks_panel.rs
│   ├── memory_panel.rs
│   ├── network_panel.rs
│   ├── processes_panel.rs
│   ├── sensors_panel.rs
│   └── system_panel.rs
└── util/
    └── format.rs
```

## Keyboard shortcuts

- `1`–`7`: switch views
- `T`: toggle theme

## Notes

- The app is focused on Linux first, but the structure avoids unnecessary Linux-only assumptions in the core model.
- GPU support is intentionally not part of the current implementation.
- Kill/signals for processes are intentionally not implemented yet.

## License

MIT
```