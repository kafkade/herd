# Herd — Roadmap

## v0.1.0 — Core Working Tool (Milestone 1) ✅

The minimum viable tool: herd windows, list displays, undo.

- [x] Project scaffolding (Cargo, deps, module structure)
- [x] Monitor enumeration (detect displays, primary, positions, DPI)
- [x] Window enumeration with filtering (visible, not minimized, not system, not cloaked)
- [x] Core herd operation (move windows to target with cascade)
- [x] CLI interface (clap derive: herd, list, undo, --display, --dry-run)
- [x] `herd list` command (display info + window counts)
- [x] Snapshot system (save before herd, restore on undo)
- [x] `--dry-run` flag (preview without moving)
- [x] DPI awareness (per-monitor v2)
- [x] Error handling (single monitor, no windows, invalid display, admin windows)
- [x] Unit tests (filtering logic, cascade math, snapshot serialization)
- [x] README.md (installation, usage, examples)

**Release**: GitHub Release with pre-built `.exe`

## v0.2.0 — Polish & Distribution (Milestone 2)

CI/CD, packaging, and developer experience.

- [ ] GitHub Actions CI (build, test, clippy, fmt check)
- [ ] Automated release workflow (tag → build → release with binary)
- [ ] winget manifest
- [ ] Scoop bucket entry
- [ ] PowerShell tab completion
- [ ] `--quiet` flag (suppress output, useful for scripts)
- [ ] `--json` flag (machine-readable output for list command)
- [ ] Landing page (GitHub Pages)

## v0.3.0 — Advanced Features (Milestone 3)

Power user features and GUI option.

- [ ] Multiple arrangement modes (cascade, tile, stack)
- [ ] `herd from <N>` — move windows FROM a specific display (to primary)
- [ ] Named profiles (`herd save work`, `herd restore work`)
- [ ] System tray app mode (`herd --tray`)
- [ ] Global hotkey registration (configurable)
- [ ] Configuration file (`%APPDATA%\herd\config.toml`)

## v0.4.0 — Automation (Milestone 4)

Event-driven and scripting features.

- [ ] Auto-herd on display disconnect
- [ ] Auto-herd on display connect (with profile)
- [ ] Windows Task Scheduler integration
- [ ] PowerShell module wrapper
- [ ] Plugin system for custom arrangement algorithms

## Non-Goals (Explicit)

- Not a general window manager (no tiling WM features)
- Not a virtual desktop manager
- No cross-platform support (Windows-only by design)
- No always-on-top management
- No window snapping (Windows already has this)
