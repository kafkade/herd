# 🐑 Herd

**Move all your windows to one display.** A fast, lightweight Windows CLI utility for multi-monitor setups.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why Herd?

If you've ever:
- Unplugged a monitor and had windows scatter to weird positions
- Changed your primary display and wanted all windows to follow
- Needed to quickly clear a screen for a presentation
- Just wanted all your windows in one place

**Herd** rounds them up for you in one command.

## Quick Start

```powershell
# Move all windows to the primary display
herd

# See your displays and window counts
herd list

# Preview what would happen (no changes made)
herd --dry-run

# Move to a specific display
herd --display 2

# Undo — restore windows to their previous positions
herd undo
```

## Installation

### Download Binary

Download the latest `herd.exe` from [Releases](https://github.com/kafkade/herd/releases) and add it to your PATH.

### Build from Source

Requires [Rust](https://rustup.rs/) 1.85+:

```powershell
git clone https://github.com/kafkade/herd.git
cd herd
cargo install --path .
```

## Usage

### `herd` — Move windows to primary display

```
$ herd
🐑 Herding windows to Display 1 (2560x1440 @ (0,0), Primary)...
✅ Herded 10 window(s) to Display 1.
```

### `herd list` — Show displays and windows

```
$ herd list
Displays:
Display    Primary    Resolution           Position             Windows
-----------------------------------------------------------------------
1          ✓          2560x1440            (0, 0)               1
2                     2560x1440            (-2560, 0)           2
3                     1920x1200            (216, 1440)          8

Total: 3 display(s), 11 window(s)
```

### `herd --display N` — Target a specific display

```
$ herd --display 2
🐑 Herding windows to Display 2 (2560x1440 @ (-2560,0))...
✅ Herded 9 window(s) to Display 2.
```

### `herd --dry-run` — Preview without moving

```
$ herd --dry-run
🐑 Herding windows to Display 1 (2560x1440 @ (0,0), Primary)...
  Would move "Outlook" from Display 2 to Display 1 at (30, 30)
  Would move "Teams" from Display 3 to Display 1 at (60, 60)
  ...
Dry run: 10 window(s) would be moved.
```

### `herd undo` — Restore previous positions

```
$ herd undo
🔄 Restoring 10 window(s)...
✅ Restored 10 window(s).
```

## CLI Reference

```
herd — Move all windows to one display

USAGE:
    herd [OPTIONS] [COMMAND]

COMMANDS:
    list        List displays and their windows
    undo        Restore windows to their previous positions
    help        Print help information

OPTIONS:
    -d, --display <N>     Target display number (default: primary)
    -n, --dry-run         Show what would happen without moving
    -h, --help            Print help
    -V, --version         Print version
```

## How It Works

1. **Enumerate displays** using Win32 `EnumDisplayMonitors` API
2. **Find target** display (primary by default, or `--display N`)
3. **Enumerate windows** using `EnumWindows`, filtering out:
   - Invisible windows
   - Minimized windows
   - System/tool windows
   - Cloaked UWP apps
4. **Save snapshot** of current positions (for undo)
5. **Cascade** windows onto the target display, preserving their sizes
6. Report results

### Window Filtering

Herd only moves "real" application windows. It automatically skips:
- Minimized windows
- System windows (taskbar, desktop, shell)
- Hidden/cloaked UWP windows
- Tool windows (floating toolbars)
- Windows already on the target display

### DPI Awareness

Herd is DPI-aware (Per-Monitor v2). It correctly handles mixed-DPI multi-monitor setups where displays have different scaling levels.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

```
src/
├── main.rs          # Entry point, DPI setup, CLI dispatch
├── cli.rs           # Clap command definitions
├── monitor.rs       # Display enumeration (Win32 GDI)
├── window.rs        # Window enumeration + filtering (Win32)
├── herd.rs          # Core orchestration (list, herd, undo)
├── snapshot.rs      # Position save/restore (JSON)
└── error.rs         # Error types
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full plan.

- [x] **v0.1.0** — Core CLI tool (herd, list, undo, dry-run)
- [ ] **v0.2.0** — CI/CD, winget/scoop packaging, shell completions
- [ ] **v0.3.0** — Arrangement modes, system tray, hotkeys, profiles
- [ ] **v0.4.0** — Auto-herd on display events, automation

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run checks: `cargo test && cargo clippy && cargo fmt -- --check`
5. Commit using [Conventional Commits](https://www.conventionalcommits.org/)
6. Open a Pull Request

## License

[MIT](LICENSE)

## Acknowledgments

- Built with [windows-rs](https://github.com/microsoft/windows-rs) — Rust bindings for the Windows API
- CLI powered by [clap](https://github.com/clap-rs/clap)
