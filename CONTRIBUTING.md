# Contributing to Herd

Thank you for your interest in contributing to Herd! This document provides guidelines and information for contributors.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior by opening an issue.

## How to Contribute

### Reporting Bugs

Before submitting a bug report:

1. **Check existing issues** to avoid duplicates.
2. **Test with the latest version** to see if the bug has already been fixed.

When filing a bug report, include:

- **Your Windows version** (e.g., Windows 11 23H2)
- **Monitor setup** (number of displays, resolutions, DPI scaling percentages)
- **Steps to reproduce** the issue
- **Expected vs. actual behavior**
- **Output of `herd list`** (helps us understand your display configuration)
- **Verbose output**: Run with `RUST_LOG=debug herd [command]` for detailed logs

### Suggesting Features

Feature requests are tracked as [GitHub Issues](https://github.com/kafkade/herd/issues). When suggesting a feature:

1. **Check the [Roadmap](ROADMAP.md)** to see if it's already planned.
2. **Open an issue** with the `enhancement` label.
3. **Describe the use case** — what problem does this solve?
4. **Propose a CLI interface** if applicable (e.g., `herd --tile` or `herd profile save work`).

### Pull Requests

1. **Open an issue first** to discuss what you'd like to change. This prevents wasted effort if the change doesn't align with the project direction.
2. **Fork the repository** and create a feature branch from `main`:
   ```powershell
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** following the [Development Guidelines](#development-guidelines) below.
4. **Write or update tests** for your changes.
5. **Ensure all checks pass** before submitting:
   ```powershell
   cargo build
   cargo test
   cargo clippy
   cargo fmt -- --check
   ```
6. **Write a clear commit message** following [Conventional Commits](https://www.conventionalcommits.org/):
   ```
   feat: add tile arrangement mode
   fix: handle DPI change during herd operation
   docs: update CLI reference with new flags
   test: add coverage for edge case with single window
   ```
7. **Open a Pull Request** against `main` with a description of what changed and why.

## Development Guidelines

### Prerequisites

- **Rust 1.85+** (install via [rustup](https://rustup.rs/))
- **Windows 10/11** (required — Herd uses Win32 APIs)
- **Multiple monitors** recommended for testing (single monitor works but limits what you can verify)

### Project Structure

```
src/
├── main.rs          # Entry point, DPI setup, CLI dispatch
├── cli.rs           # Clap command/argument definitions
├── monitor.rs       # Display enumeration (Win32 GDI)
├── window.rs        # Window enumeration, filtering, positioning
├── herd.rs          # Core orchestration (herd, list, undo commands)
├── snapshot.rs      # Position save/restore (JSON)
└── error.rs         # Custom error types
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

### Building

```powershell
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run -- list
cargo run -- --dry-run
```

### Testing

```powershell
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run a specific test
cargo test test_cascade_basic
```

**Note**: Some functionality (actual window moving) can only be tested manually on a multi-monitor setup. Unit tests cover the algorithmic logic (cascade math, filtering predicates, snapshot serialization).

### Linting

```powershell
# Run clippy
cargo clippy

# Check formatting
cargo fmt -- --check

# Auto-format
cargo fmt
```

### Code Style

- Follow existing patterns in the codebase.
- Use `anyhow::Result` for error propagation in command functions.
- Use `log::debug!` / `log::warn!` / `log::info!` for diagnostic output — not `println!` (except for user-facing output).
- All Win32 API calls must be wrapped in `unsafe` blocks with appropriate safety comments when non-obvious.
- Prefer extending existing modules over creating new ones.

### Win32 API Guidelines

When working with Win32 APIs:

- **Always check return values**. Don't discard `BOOL` or `Result` returns.
- **Handle `HWND` lifetime carefully**. Window handles can become invalid at any time — validate before use.
- **Use `SWP_NOSIZE` intentionally**. The herd operation preserves size; undo restores it. Don't mix these up.
- **Test with different DPI scales**. Mixed-DPI bugs are subtle and common.
- **Filter defensively**. It's better to skip an unusual window than to crash or move a system window.

### Adding a New Command

1. Add the variant to `Commands` enum in `src/cli.rs`
2. Add the handler function in `src/herd.rs`
3. Add the `match` arm in `src/main.rs`
4. Update `README.md` and `CHANGELOG.md`

### Adding a New Window Filter

1. Add the filter check in `enum_windows_callback` in `src/window.rs`
2. Add the class name to `SKIP_CLASSES` if filtering by class
3. Document the filter in `docs/ARCHITECTURE.md` (filtering pipeline section)

## Release Process

Releases follow [Semantic Versioning](https://semver.org/):

- **PATCH** (0.1.x): Bug fixes, documentation improvements
- **MINOR** (0.x.0): New features, new commands, non-breaking changes
- **MAJOR** (x.0.0): Breaking CLI changes, removed commands

Each release should:

1. Update `CHANGELOG.md` with all changes
2. Update version in `Cargo.toml`
3. Tag: `git tag -a v0.x.0 -m "Release v0.x.0"`
4. Build release binary: `cargo build --release`
5. Create GitHub Release with the binary and changelog entry

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/kafkade/herd/discussions) or an issue tagged `question`
- **Bugs**: File an [Issue](https://github.com/kafkade/herd/issues) with the `bug` label
- **Features**: File an [Issue](https://github.com/kafkade/herd/issues) with the `enhancement` label

## License

By contributing to Herd, you agree that your contributions will be licensed under the [MIT License](LICENSE).
