mod args;

use args::{Args, Command};
use clap::Parser;
use injectra_core::{Engine, Result};

fn run(args: Args) -> Result<serde_json::Value> {
    match args.command {
        Command::Targets { port } => Ok(serde_json::to_value(
            injectra_cdp::CdpEngine::new(port)?.targets()?,
        )?),
        Command::Apply {
            port,
            target,
            bundle,
        } => {
            let program = injectra_payload::apply(&injectra_payload::Bundle::load(&bundle)?)?;
            execute(port, &target, &program)
        }
        Command::Remove { port, target, id } => {
            execute(port, &target, &injectra_payload::remove(&id)?)
        }
        Command::LaunchPlan { app, port } => Ok(serde_json::to_value(
            injectra_platform_macos::launch_plan(&app, port)?,
        )?),
    }
}

fn execute(port: u16, target: &str, program: &str) -> Result<serde_json::Value> {
    // Composition root: this is the only place selecting the concrete runtime adapter.
    let engine = injectra_cdp::CdpEngine::new(port)?;
    let mut session = engine.connect(target)?;
    injectra_core::execute(session.as_mut(), program)
}

fn main() -> std::process::ExitCode {
    match run(Args::parse()) {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("JSON value serializes")
            );
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("injectra: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
