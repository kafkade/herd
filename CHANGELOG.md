# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-25

### Added

- **Core herd command**: Move all visible windows to a target display with cascade positioning.
- **Display targeting**: Default to primary display, or specify with `--display N`.
- **`herd list` command**: Show all displays with resolution, position, and window counts.
- **`herd undo` command**: Restore windows to their previous positions from a saved snapshot.
- **`--dry-run` flag**: Preview which windows would be moved without making changes.
- **Window filtering pipeline**: Automatically skips invisible, minimized, system, cloaked (UWP), tool windows, and shell windows (Progman, WorkerW, Shell_TrayWnd).
- **Snapshot system**: Saves window positions to `%LOCALAPPDATA%\herd\snapshots\` before each herd operation. FIFO pruning keeps last 10 snapshots.
- **HWND identity validation on undo**: Verifies window title matches before restoring to prevent moving wrong windows when handles are reused.
- **Per-monitor DPI v2 awareness**: Correctly handles mixed-DPI multi-monitor setups.
- **Mixed-DPI undo support**: Restores both position and size (not just position) to handle DPI-induced resizing.
- **Graceful error handling**: Friendly messages for single-monitor setups, no windows to move, invalid display numbers, and admin-elevated windows.
- **Unit tests**: Coverage for cascade positioning, monitor targeting, rect math, and snapshot serialization.

### Technical Details

- Built with Rust using the `windows` crate (v0.61) for Win32 API bindings.
- CLI powered by `clap` v4 with derive API.
- Snapshots stored as JSON via `serde`/`serde_json`.

[Unreleased]: https://github.com/kafkade/herd/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kafkade/herd/releases/tag/v0.1.0
