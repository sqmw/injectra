mod args;
mod backend;
mod studio;
use args::{Args, Command};
use clap::Parser;
use injectra_core::{Result, TargetRef};
use serde_json::{Value, json};

fn run(args: Args) -> Result<Value> {
    match args.command {
        Command::Create { workspace, id } => {
            let w = injectra_workspace::Workspace::create(&workspace, &id)?;
            Ok(json!({"id":w.id(),"root":w.root()}))
        }
        Command::Import {
            workspace,
            manifest,
        } => {
            let w = injectra_workspace::Workspace::import_manifest(&workspace, &manifest)?;
            Ok(json!({"id":w.id(),"root":w.root()}))
        }
        Command::Prepare {
            workspace,
            app,
            data_mode,
            cache,
        } => studio::request(
            &workspace,
            cache,
            "prepare",
            json!({"app":app,"data_mode":data_mode}),
        ),
        Command::Dev { workspace, cache } => studio::request(&workspace, cache, "dev", json!({})),
        Command::Export {
            workspace,
            destination,
            target,
            generation,
            path_suffix,
            origin,
            cache,
        } => studio::request(
            &workspace,
            cache,
            "export",
            json!({"destination":destination,"target":target,"generation":generation,"path_suffix":path_suffix,"origin":origin}),
        ),
        Command::Studio { .. } => Err("Studio must run through the service entry".into()),
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
    let args = Args::parse();
    let command = match args.command {
        Command::Studio {
            workspace,
            cache,
            assets,
        } => return studio::serve(workspace, cache, assets),
        command => command,
    };
    match run(Args { command }) {
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
