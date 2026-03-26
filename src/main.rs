mod cli;
mod error;
mod herd;
mod monitor;
mod snapshot;
mod version;
mod window;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    env_logger::init();

    // Set DPI awareness before any Win32 calls
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        if let Err(e) = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) {
            log::warn!(
                "Could not set per-monitor DPI awareness: {}. \
                 Window positions may be inaccurate on mixed-DPI setups.",
                e
            );
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::List) => herd::list_displays()?,
        Some(Commands::Undo) => herd::undo()?,
        Some(Commands::Version) => version::print_version(),
        None => herd::run(cli.display, cli.dry_run)?,
    }

    Ok(())
}
