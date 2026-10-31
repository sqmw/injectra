use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Injectra: local Web UI customization for accessible Electron pages"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Discover page targets on an explicitly enabled loopback debugging port.
    Targets {
        #[arg(long)]
        port: u16,
    },
    /// Apply a trusted local bundle to exactly one page (no automatic reinjection).
    Apply {
        #[arg(long)]
        port: u16,
        #[arg(long)]
        target: String,
        #[arg(long)]
        bundle: PathBuf,
    },
    /// Remove managed CSS/HTML and run cleanup callbacks registered by the script.
    Remove {
        #[arg(long)]
        port: u16,
        #[arg(long)]
        target: String,
        #[arg(long)]
        id: String,
    },
    /// Inspect a standard macOS Electron .app and print a launch plan; never starts it.
    LaunchPlan {
        #[arg(long)]
        app: PathBuf,
        #[arg(long)]
        port: u16,
    },
}
