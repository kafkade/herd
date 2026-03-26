# Copilot Instructions for Herd

## What is Herd?

A Windows-only Rust CLI that moves visible application windows onto a single monitor. It uses Win32 APIs directly via the `windows` crate. This is **not** a general window manager — it solves one problem well.

## Build, Test, Lint

```powershell
cargo build                       # Debug build
cargo build --release             # Release build
cargo test                        # All tests
cargo test test_cascade_basic     # Single test by name
cargo test -- --nocapture         # Tests with stdout
cargo clippy                      # Lint
cargo fmt -- --check              # Format check
cargo fmt                         # Auto-format
cargo run -- list                 # Run a command directly
cargo run -- --dry-run            # Preview herd without moving windows
RUST_LOG=debug cargo run -- list  # Verbose debug logging
```

All four checks must pass before committing: `cargo build && cargo test && cargo clippy && cargo fmt -- --check`

## Architecture

**Data flow for `herd` (default command):**
DPI setup → enumerate monitors → select target → enumerate/filter windows → save snapshot → compute cascade positions → move windows → report

**Module responsibilities:**

- `main.rs` — Entry point. Sets per-monitor DPI v2 awareness, inits `env_logger`, parses CLI, dispatches to `herd::run`, `herd::list_displays`, or `herd::undo`.
- `cli.rs` — Clap derive definitions. `Cli` struct (with `--display` and `--dry-run`) and `Commands` enum (`List`, `Undo`). Running with no subcommand performs the herd operation.
- `herd.rs` — Orchestration. The three public entry points (`run`, `list_displays`, `undo`) coordinate monitors, windows, and snapshots. Friendly user-facing messages are printed here; most error cases resolve to a clean message + `Ok(())` rather than propagating errors.
- `monitor.rs` — `MonitorInfo` and `Rect` types. Enumerates via `EnumDisplayMonitors`/`GetMonitorInfoW`. Monitors are sorted primary-first, then left-to-right/top-to-bottom, and assigned 1-based indices.
- `window.rs` — `WindowInfo` type. Enumerates via `EnumWindows` with a multi-stage filtering pipeline (visibility → minimized → extended styles → cloaked → title → class blacklist → rect). Also provides `cascade_positions` (30px diagonal offset, wrap with 15px shift) and `move_window` (wraps `SetWindowPos`).
- `snapshot.rs` — `Snapshot`/`WindowSnapshot` types. Saves JSON to `%LOCALAPPDATA%\herd\snapshots\{timestamp_millis}.json`. Max 10 snapshots with FIFO pruning. Undo validates HWND identity by comparing current window title to snapshot.
- `error.rs` — `HerdError` enum (`SingleMonitor`, `InvalidDisplay`, `NoWindowsToMove`, `NoSnapshot`). In practice, most commands use friendly prints rather than these errors.

## Git Policy

- **Never commit automatically.** Do not run `git commit`, `git push`, or any
  other command that creates or modifies commits without explicit user approval.
- Always present proposed changes and let the user decide when to commit.
- This applies to all agents, sub-agents, and automated workflows.

## Key Conventions

### Error Handling
- Use `anyhow::Result` for all command functions. Orchestration errors are printed as friendly messages and return `Ok(())`; only truly unexpected failures bubble up.
- Per-window failures during herd/undo are logged as warnings and skipped — the operation continues.

### Logging
- Use `log::debug!` / `log::info!` / `log::warn!` for diagnostics — **never `println!`** except for intentional user-facing output.
- Enable with `RUST_LOG=debug`.

### Win32 API Rules
- All Win32 calls go in `unsafe` blocks with safety comments when non-obvious.
- Always check return values from Win32 APIs (`BOOL`, `Result`).
- `HWND` handles are stored as `isize` and converted only at the Win32 call site.
- Window handles can become invalid at any time — validate before use.
- Use `SWP_NOSIZE` for herd (preserve size), restore both position and size for undo.
- Filter defensively: better to skip an unusual window than crash or move a system window.

### Adding a New Command
1. Add variant to `Commands` in `src/cli.rs`
2. Add handler function in `src/herd.rs`
3. Add `match` arm in `src/main.rs`
4. Update `README.md` and `CHANGELOG.md`

### Adding a New Window Filter
1. Add check in the `EnumWindows` callback in `src/window.rs`
2. Add class name to the blacklist if filtering by class
3. Document in `docs/ARCHITECTURE.md`

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `test:`, etc.

### Testing Limitations
Unit tests cover algorithmic logic (cascade math, filtering predicates, snapshot serialization). Actual window movement can only be tested manually on a multi-monitor Windows setup.

### Platform
Windows-only. Requires Rust 1.85+ and the `windows` crate with specific Win32 feature gates (listed in `Cargo.toml`).
