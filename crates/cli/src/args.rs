use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    version,
    about = "Injectra: runtime Web code injection into accessible pages"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Backend {
    Cdp,
    ElectronGadget,
    Webkit,
}
#[derive(ClapArgs)]
pub struct Endpoint {
    /// Defaults to CDP when omitted.
    #[arg(long, value_enum)]
    pub backend: Option<Backend>,
    #[arg(
        long,
        required_if_eq("backend", "cdp"),
        required_unless_present = "session",
        conflicts_with = "session"
    )]
    pub port: Option<u16>,
    #[arg(
        long,
        required_if_eq_any([("backend", "electron-gadget"),("backend", "webkit")]),
        requires = "backend",
        conflicts_with = "port"
    )]
    pub session: Option<PathBuf>,
}
#[derive(Subcommand)]
pub enum Command {
    /// Create a new external increment workspace.
    Create {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        id: String,
    },
    /// Import a manifest into a new or empty external workspace.
    Import {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
    },
    /// Run the authenticated local workbench for an external workspace.
    Studio {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        cache: Option<PathBuf>,
        #[arg(long)]
        assets: Option<PathBuf>,
    },
    /// Ask the running Studio to prepare an application copy.
    Prepare {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        app: PathBuf,
        #[arg(long,default_value="isolated",value_parser=["isolated","shared"])]
        data_mode: String,
        #[arg(long)]
        cache: Option<PathBuf>,
    },
    /// Launch the current prepared development artifact through Studio.
    Dev {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        cache: Option<PathBuf>,
    },
    /// Export the current verified increment snapshot through Studio.
    Export {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        destination: PathBuf,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        generation: Option<String>,
        #[arg(long)]
        path_suffix: Option<String>,
        #[arg(long)]
        origin: Option<String>,
        #[arg(long)]
        cache: Option<PathBuf>,
    },
    /// Discover main pages using the explicitly selected backend.
    Targets {
        #[command(flatten)]
        endpoint: Endpoint,
    },
    /// Apply a trusted local bundle to exactly one page (no automatic reinjection).
    Apply {
        #[command(flatten)]
        endpoint: Endpoint,
        #[arg(long)]
        target: String,
        #[arg(long)]
        bundle: PathBuf,
    },
    /// Remove managed nodes and execute registered cleanup callbacks.
    Remove {
        #[command(flatten)]
        endpoint: Endpoint,
        #[arg(long)]
        target: String,
        #[arg(long)]
        id: String,
    },
    /// Print a macOS Electron launch plan; never starts an application.
    LaunchPlan {
        #[arg(long)]
        app: PathBuf,
        #[arg(long)]
        port: u16,
    },
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backend_arguments_are_explicit_and_legacy_port_still_works() {
        assert!(Args::try_parse_from(["injectra", "targets", "--port", "9222"]).is_ok());
        assert!(Args::try_parse_from(["injectra", "targets"]).is_err());
        assert!(
            Args::try_parse_from([
                "injectra",
                "targets",
                "--backend",
                "webkit",
                "--session",
                "s.json"
            ])
            .is_ok()
        );
        assert!(
            Args::try_parse_from([
                "injectra",
                "targets",
                "--backend",
                "webkit",
                "--port",
                "9222"
            ])
            .is_err()
        );
        assert!(
            Args::try_parse_from([
                "injectra",
                "targets",
                "--backend",
                "electron-gadget",
                "--session",
                "s.json"
            ])
            .is_ok()
        );
        assert!(
            Args::try_parse_from([
                "injectra",
                "targets",
                "--backend",
                "electron-gadget",
                "--port",
                "9222"
            ])
            .is_err()
        );
        assert!(Args::try_parse_from(["injectra", "targets", "--session", "s.json"]).is_err());
    }
}
