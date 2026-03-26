# Herd — Architecture

## Overview

Herd is a Windows CLI utility that moves all visible windows to a single display. Built in Rust with direct Win32 API calls for minimal overhead and instant startup.

## System Architecture

```
┌───────────────────────────────────────────────────┐
│                    CLI Layer                      │
│  main.rs → clap parsing → command dispatch        │
├───────────────────────────────────────────────────┤
│                 Orchestration                     │
│  herd.rs → coordinates monitors + windows         │
├──────────────────┬────────────────────────────────┤
│   Monitor Layer  │      Window Layer              │
│  monitor.rs      │  window.rs                     │
│  - enumerate     │  - enumerate                   │
│  - get info      │  - filter (visible, not        │
│  - find primary  │    minimized, not system)      │
│  - find by index │  - move / set position         │
├──────────────────┴────────────────────────────────┤
│                 Snapshot Layer                    │
│  snapshot.rs → save/restore positions to JSON     │
├───────────────────────────────────────────────────┤
│              Win32 API (windows crate)            │
│  EnumDisplayMonitors, EnumWindows, SetWindowPos   │
│  GetMonitorInfoW, DwmGetWindowAttribute, etc.     │
└───────────────────────────────────────────────────┘
```

## Data Flow

### Herd Operation

```
1. Set DPI awareness (per-monitor v2)
2. Enumerate monitors → find target (primary or --display N)
3. Enumerate windows → apply filtering rules
4. Save snapshot (pre-herd positions)
5. Calculate cascade positions on target monitor
6. Move each window via SetWindowPos
7. Report results
```

### Undo Operation

```
1. Read most recent snapshot from %LOCALAPPDATA%\herd\snapshots\
2. For each saved window: validate HWND identity via title match
3. Restore original position and size via SetWindowPos
4. Delete consumed snapshot
```

## Window Filtering Pipeline

```
All top-level windows (EnumWindows)
  │
  ├─ IsWindowVisible? ──── NO ──→ skip
  │
  ├─ IsIconic? ─────────── YES ─→ skip (minimized)
  │
  ├─ WS_EX_TOOLWINDOW? ── YES ─→ skip (floating toolbars)
  │
  ├─ WS_EX_NOACTIVATE? ── YES ─→ skip (non-interactive)
  │
  ├─ DWMWA_CLOAKED? ────── YES ─→ skip (hidden UWP)
  │
  ├─ Empty title? ──────── YES ─→ skip (system invisible)
  │
  ├─ Shell class? ──────── YES ─→ skip (Progman, WorkerW,
  │                                  Shell_TrayWnd, etc.)
  │
  ├─ Already on target? ── YES ─→ skip (no-op)
  │
  └─ ✓ Herdable window
```

### Filtered Window Classes

The following Win32 window classes are explicitly skipped:

| Class                                 | Description                  |
| ------------------------------------- | ---------------------------- |
| `Progman`                             | Desktop / Program Manager    |
| `WorkerW`                             | Desktop background worker    |
| `Shell_TrayWnd`                       | Taskbar (primary)            |
| `Shell_SecondaryTrayWnd`              | Taskbar (secondary monitors) |
| `Windows.UI.Core.CoreWindow`          | UWP core window              |
| `XamlExplorerHostIslandWindow`        | XAML host island             |
| `TopLevelWindowForOverflowXamlIsland` | XAML overflow island         |

## Cascade Positioning

```
Monitor work area (excludes taskbar):
┌─────────────────────────────────────┐
│ ┌─────────────┐                     │
│ │  Window 1   │                     │
│ │  ┌─────────────┐                  │
│ │  │  Window 2   │                  │
│ │  │  ┌─────────────┐               │
│ └──│  │  Window 3   │               │
│    │  │             │               │
│    └──│             │               │
│       └─────────────┘               │
│                                     │
└─────────────────────────────────────┘

Offset: 30px horizontal, 30px vertical per window
Wrap: when next position would place >50% of window outside work area
Wrap offset: each wrap cycle shifts start position by 15px to prevent stacking
```

## Snapshot Format

```json
{
  "version": 1,
  "timestamp": "2026-03-25T16:30:00Z",
  "target_display": 1,
  "windows": [
    {
      "hwnd": 12345678,
      "title": "Visual Studio Code",
      "rect": { "left": 100, "top": 50, "right": 1380, "bottom": 818 },
      "monitor_index": 2
    }
  ]
}
```

**Storage**: `%LOCALAPPDATA%\herd\snapshots\{unix_timestamp_ms}.json`

**Retention**: Max 10 snapshots (FIFO pruning after each save)

**Undo safety**:

- HWND identity is validated via title match before restoring (prevents moving wrong windows if handles are reused)
- Snapshot is only deleted after restore attempt completes (allows retry on partial failure)
- Both position and size are restored (important for mixed-DPI setups)

## DPI Handling

The app sets per-monitor DPI awareness v2 at startup via `SetProcessDpiAwarenessContext`. This ensures:

- Window coordinates are in physical pixels, not scaled
- Moving windows between monitors with different DPI works correctly
- No coordinate translation needed between monitors
- Undo correctly restores original dimensions after DPI-induced resize

If the DPI awareness call fails (e.g., already set by a manifest), a warning is logged and the tool continues with potentially virtualized coordinates.

## Error Handling Strategy

| Scenario                | Behavior                                                        |
| ----------------------- | --------------------------------------------------------------- |
| Single monitor          | Print friendly message, exit 0                                  |
| No herdable windows     | Print "All windows are already on Display N", exit 0            |
| Invalid display number  | Print available displays, exit 1                                |
| Can't move admin window | Warn per-window, continue with others, suggest running as admin |
| SetWindowPos fails      | Log warning, continue with remaining windows                    |
| No snapshot for undo    | Print "No previous herd operation to undo", exit 0              |
| HWND reused (undo)      | Skip window with warning, continue with others                  |
| Window closed (undo)    | Skip window with warning, continue with others                  |
