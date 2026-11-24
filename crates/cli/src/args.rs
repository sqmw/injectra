use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    version,
    about = "Injectra: runtime Web injection and pre-runtime CLI dev"
)]
pub struct Args {
    /// Emit single-line structured JSON envelopes and certainty-mapped exit codes.
    #[arg(long, global = true)]
    pub machine: bool,
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
    /// Prepare a derived Electron app with a small dev entry.
    Prepare {
        #[arg(long)]
        app: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long,value_parser=["shared","isolated"])]
        data_mode: String,
    },
    /// Watch external page increments and apply them in the real development app.
    Dev {
        #[arg(long)]
        artifact: PathBuf,
        #[arg(long)]
        bundle: PathBuf,
        #[arg(long)]
        target: Option<String>,
        /// Explicitly allow a new attempt after this unresolved request.
        #[arg(long)]
        acknowledge_unknown: Option<String>,
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
    /// List managed bundles currently registered in the selected page.
    List {
        #[command(flatten)]
        endpoint: Endpoint,
        #[arg(long)]
        target: String,
    },
    /// Evaluate a JavaScript expression in the selected page context (trusted local code).
    Eval {
        #[command(flatten)]
        endpoint: Endpoint,
        #[arg(long)]
        target: String,
        #[arg(long, required_unless_present = "file", conflicts_with = "file")]
        expression: Option<String>,
        #[arg(
            long,
            required_unless_present = "expression",
            conflicts_with = "expression"
        )]
        file: Option<PathBuf>,
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

    #[test]
    fn machine_flag_and_observation_subcommands_parse_as_specified() {
        let before = Args::try_parse_from(["injectra", "--machine", "targets", "--port", "9222"])
            .expect("global --machine before subcommand");
        assert!(before.machine);

        let after = Args::try_parse_from(["injectra", "targets", "--port", "9222", "--machine"])
            .expect("global --machine after subcommand");
        assert!(after.machine);

        assert!(
            Args::try_parse_from(["injectra", "list", "--port", "9222", "--target", "p1"]).is_ok()
        );
        assert!(
            Args::try_parse_from([
                "injectra",
                "list",
                "--backend",
                "webkit",
                "--session",
                "s.json",
                "--target",
                "p1"
            ])
            .is_ok()
        );
        assert!(
            Args::try_parse_from([
                "injectra",
                "eval",
                "--port",
                "9222",
                "--target",
                "p1",
                "--expression",
                "document.title"
            ])
            .is_ok()
        );
        assert!(
            Args::try_parse_from([
                "injectra", "eval", "--port", "9222", "--target", "p1", "--file", "probe.js"
            ])
            .is_ok()
        );
        assert!(
            Args::try_parse_from(["injectra", "eval", "--port", "9222", "--target", "p1"]).is_err()
        );
        assert!(
            Args::try_parse_from([
                "injectra",
                "eval",
                "--port",
                "9222",
                "--target",
                "p1",
                "--expression",
                "1",
                "--file",
                "probe.js"
            ])
            .is_err()
        );
    }
}
