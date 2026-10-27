# Architecture

## Overview

HiraraMonitor uses a simple two-layer design:

1. **Collector layer**: gathers system metrics in a background thread.
2. **UI layer**: renders the latest metrics with egui.

This keeps `sysinfo` refresh calls out of the render loop.

## Main modules

- `src/main.rs`: application entry point.
- `src/app.rs`: application state, theme, metrics ingestion, layout and routing.
- `src/config.rs`: in-memory configuration and CLI arguments.
- `src/metrics/snapshot.rs`: immutable data models used by all views.
- `src/metrics/collector.rs`: background collector and `SystemSnapshot` creation.
- `src/metrics/history.rs`: fixed-capacity history buffers.
- `src/platform`: platform-specific helpers. Linux disk throughput uses `/proc/diskstats`.
- `src/ui`: individual panels for each view.
- `src/util/format.rs`: formatting helpers.

## Data flow

```text
sysinfo::System
       |
       v
Collector thread
       |
       v
Arc<SystemSnapshot>
       |
       v
mpsc channel
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

## Metrics model

The collector produces one `SystemSnapshot` per refresh cycle. Views do not mutate the snapshot; they render from it.

Important structures:

- `SystemSnapshot`
- `CpuCoreSnapshot`
- `MemorySnapshot`
- `DiskSnapshot`
- `NetworkSnapshot`
- `ProcessSnapshot`
- `ComponentSnapshot`
- `SystemMetadata`

## History handling

- `HistoryBuffer` is a fixed-size circular buffer.
- CPU, memory, swap and network views use rolling 60-second viewports.
- The X axis uses wall-clock epoch time formatted as `HH:MM:SS`.

## Process view

- Filtering happens before pagination.
- Sorting can use CPU, memory, name or PID.
- The selected process gets a detail panel.
- Pagination controls are placed above the table.
- Columns use fixed widths to avoid unstable layouts.

## Platform behavior

- Linux disk throughput is enriched with `/proc/diskstats`.
- Non-Linux targets currently use the unsupported platform fallback.
- No global configuration files are written. Settings remain in memory or explicit CLI input.

## Performance considerations

- Metric collection runs in a separate thread.
- The UI uses `try_recv()` to avoid blocking.
- Scrollable views use `ScrollArea`.
- Process tables use fixed-width cells and pagination.
- Graphs use rolling bounds and disabled drag/zoom to keep rendering stable.
```