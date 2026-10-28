# Architecture

## Overview

HiraraMonitor uses a focused two-layer design:

1. **Collector layer**: gathers system metrics on a background thread.
2. **UI layer**: renders the latest metrics with egui.

The collector produces one immutable `SystemSnapshot` per refresh. Panels render from a shared `Arc<SystemSnapshot>` without mutating it.

## Main modules

- `src/main.rs`: entry point, logging initialization and viewport creation.
- `src/app.rs`: application state, theme application, routing, metric ingestion and persisted preferences.
- `src/config.rs`: persisted configuration types and validation.
- `src/config_store.rs`: TOML loading, atomic saving and CLI-compatible defaults.
- `src/metrics/collector.rs`: background collector, error handling and backoff.
- `src/metrics/history.rs`: fixed-capacity circular history buffers.
- `src/metrics/snapshot.rs`: immutable data models used by every panel.
- `src/platform/`: Linux-specific and unsupported helpers.
- `src/ui/`: panels, reusable components and visual theme.
- `src/util/`: formatting helpers.

## Data flow

```text
sysinfo + platform files
       |
       v
Collector thread
       |
       v
Arc<SystemSnapshot>
       |
       v
mpsc::sync_channel(1)
       |
       v
HiraraApp::receive_metrics
       |
       v
Latest snapshot + HistoryBuffer
       |
       v
egui panels
```

## Process handling

- Process rows use stable fixed widths.
- Sorting is configurable by PID, name, CPU, memory, state and user.
- Selection supports single and multi-select.
- Batch terminate and kill require confirmation.
- Process details render in a floating `egui::Window`.
- `pid == 1` and the application PID are protected from signals.

## Configuration

The app stores compact `PersistedConfig` data:

- theme
- initial view
- refresh interval
- history length
- process page size and sort
- visual alert thresholds
- window size when captured

Load order:

1. defaults
2. `--config <path>` or `HIRARA_CONFIG`
3. default `~/.config/hiraramonitor/config.toml` on Linux
4. CLI flags

Preferences are saved at close and when a changed preference is debounced.

## Collector robustness

- Collection is split from the worker loop.
- Targeted refreshes replace `refresh_all()` where practical.
- Failed collection is logged and retries with backoff.
- Snapshots can be dropped locally under pressure, but UI remains alive.
- Panic reporting is written to the shared collector error state.

## Performance and visualization

- Theme is only applied when it changes.
- UI repaint is limited to the configured refresh interval, capped at 250 ms.
- Process, disk and sensor tables avoid unstable width calculations.
- Rolling graphs use fixed bounded X axes.
- Visual alert colors are computed from persisted thresholds.

## Platform behavior

- Linux disk throughput is read from `/proc/diskstats`.
- Unsupported targets receive empty disk throughput.
- Process signal support comes from `sysinfo` on Linux.
- No application code relies on a mandatory global configuration file; persistence uses explicit or default paths.
