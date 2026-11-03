mod args;
use args::{Args, Command};
use clap::Parser;
use injectra_core::{Result, TargetRef};
use injectra_web::WebBackend;
use serde_json::{Value, json};

fn run(args: Args) -> Result<Value> {
    match args.command {
        Command::Targets { port } => {
            let backend = injectra_cdp::CdpBackend::new(port)?;
            let targets = injectra_application::targets(&backend)?;
            // Preserve the v0.1 CLI wire format, without serializing internal identity.
            Ok(Value::Array(
                targets
                    .into_iter()
                    .map(|t| json!({"id":t.reference.key(),"title":t.title,"url":t.url}))
                    .collect(),
            ))
        }
        Command::Apply {
            port,
            target,
            bundle,
        } => {
            let backend = injectra_cdp::CdpBackend::new(port)?;
            injectra_application::apply(&backend, &TargetRef::new(backend.id(), target), &bundle)
        }
        Command::Remove { port, target, id } => {
            let backend = injectra_cdp::CdpBackend::new(port)?;
            injectra_application::remove(&backend, &TargetRef::new(backend.id(), target), &id)
        }
        Command::LaunchPlan { app, port } => Ok(serde_json::to_value(
            injectra_hosts::electron::macos_launch_plan(&app, port)?,
        )
        .expect("launch plan serializes")),
    }
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
