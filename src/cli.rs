use clap::{Parser, Subcommand};

/// Move all windows to one display
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = false)]
pub struct Cli {
    /// Target display number (default: primary)
    #[arg(short, long)]
    pub display: Option<u32>,

    /// Show what would happen without moving windows
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List displays and their windows
    List,
    /// Restore windows to their previous positions
    Undo,
    /// Show version with animated logo
    Version,
}
