mod args;
mod backend;
mod dev;
use args::{Args, Command};
use clap::Parser;
use injectra_core::{Result, TargetRef};
use serde_json::{Value, json};

fn run(args: Args) -> Result<Value> {
    match args.command {
        Command::Prepare {
            app,
            output,
            data_mode,
        } => {
            let data_mode = if data_mode == "shared" {
                injectra_preparation::DataMode::Shared
            } else {
                injectra_preparation::DataMode::Isolated
            };
            Ok(serde_json::to_value(injectra_preparation::prepare(
                &injectra_preparation::PrepareRequest {
                    source_app: app,
                    output_root: output,
                    data_mode,
                },
            )?)
            .expect("artifact serializes"))
        }
        Command::Dev {
            artifact,
            bundle,
            target,
            acknowledge_unknown,
        } => dev::run(
            &artifact,
            &bundle,
            target.as_deref(),
            acknowledge_unknown.as_deref(),
        ),
        Command::Targets { endpoint } => {
            let backend = backend::create(endpoint)?;
            let targets = injectra_application::targets(backend.as_ref())?;
            // Preserve the v0.1 CLI wire format, without serializing internal identity.
            Ok(Value::Array(
                targets
                    .into_iter()
                    .map(|t| json!({"id":t.reference.key(),"title":t.title,"url":t.url}))
                    .collect(),
            ))
        }
        Command::Apply {
            endpoint,
            target,
            bundle,
        } => {
            let backend = backend::create(endpoint)?;
            injectra_application::apply(
                backend.as_ref(),
                &TargetRef::new(backend.id(), target),
                &bundle,
            )
        }
        Command::Remove {
            endpoint,
            target,
            id,
        } => {
            let backend = backend::create(endpoint)?;
            injectra_application::remove(
                backend.as_ref(),
                &TargetRef::new(backend.id(), target),
                &id,
            )
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
