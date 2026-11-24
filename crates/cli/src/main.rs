mod args;
mod backend;
mod dev;
use args::{Args, Command};
use clap::Parser;
use injectra_core::{Certainty, Error, ErrorKind, Phase, Result, TargetRef};
use serde_json::{Value, json};
use std::{ffi::OsStr, fs, io::Read, path::PathBuf, process::ExitCode};

const EVAL_LIMIT: u64 = 1024 * 1024;

fn read_eval_source(expression: Option<String>, file: Option<PathBuf>) -> Result<String> {
    if let Some(expr) = expression {
        return Ok(expr);
    }
    let path = file.ok_or("either --expression or --file is required")?;
    let mut text = String::new();
    fs::File::open(path)?
        .take(EVAL_LIMIT + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > EVAL_LIMIT {
        return Err("evaluation file exceeds 1 MiB".into());
    }
    Ok(text)
}

fn has_machine_flag<I, T>(args: I) -> bool
where
    I: IntoIterator<Item = T>,
    T: AsRef<OsStr>,
{
    args.into_iter()
        .skip(1)
        .take_while(|arg| arg.as_ref() != "--")
        .any(|arg| arg.as_ref() == "--machine")
}

fn machine_exit_code(certainty: Certainty) -> ExitCode {
    match certainty {
        Certainty::NotExecuted => ExitCode::from(3),
        Certainty::Executed => ExitCode::from(4),
        Certainty::Unknown => ExitCode::from(5),
    }
}

fn machine_error_envelope(
    kind: ErrorKind,
    phase: Phase,
    certainty: Certainty,
    message: &str,
    causes: Vec<String>,
) -> Value {
    json!({
        "ok": false,
        "error": {
            "kind": kind.as_str(),
            "phase": phase.as_str(),
            "certainty": certainty.as_str(),
            "message": message,
            "causes": causes,
        }
    })
}

fn machine_ok_envelope(data: Value) -> Value {
    json!({ "ok": true, "data": data })
}

fn format_machine_error(error: &Error) -> Value {
    machine_error_envelope(
        error.kind,
        error.phase,
        error.certainty,
        &error.to_string(),
        error.causes(),
    )
}

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
        Command::List { endpoint, target } => {
            let backend = backend::create(endpoint)?;
            injectra_application::list(backend.as_ref(), &TargetRef::new(backend.id(), target))
        }
        Command::Eval {
            endpoint,
            target,
            expression,
            file,
        } => {
            let source = read_eval_source(expression, file)?;
            let backend = backend::create(endpoint)?;
            injectra_application::eval(
                backend.as_ref(),
                &TargetRef::new(backend.id(), target),
                &source,
            )
        }
        Command::LaunchPlan { app, port } => Ok(serde_json::to_value(
            injectra_hosts::electron::macos_launch_plan(&app, port)?,
        )
        .expect("launch plan serializes")),
    }
}

fn print_single_line(value: &Value) {
    println!(
        "{}",
        serde_json::to_string(value).expect("JSON value serializes")
    );
}

fn main() -> ExitCode {
    let machine = has_machine_flag(std::env::args_os());
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(clap_err) => {
            if !clap_err.use_stderr() {
                let _ = clap_err.print();
                return ExitCode::SUCCESS;
            }
            if machine {
                let envelope = machine_error_envelope(
                    ErrorKind::InvalidInput,
                    Phase::Prepare,
                    Certainty::NotExecuted,
                    clap_err.to_string().trim_end(),
                    Vec::new(),
                );
                print_single_line(&envelope);
            } else {
                let _ = clap_err.print();
            }
            return ExitCode::from(2);
        }
    };
    let use_machine = args.machine;
    match run(args) {
        Ok(value) => {
            if use_machine {
                print_single_line(&machine_ok_envelope(value));
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).expect("JSON value serializes")
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            if use_machine {
                print_single_line(&format_machine_error(&error));
                machine_exit_code(error.certainty)
            } else {
                eprintln!("injectra: {error}");
                ExitCode::FAILURE
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_machine_flag_ignores_binary_name_and_double_dash_tail() {
        assert!(has_machine_flag(["injectra", "--machine", "targets"]));
        assert!(has_machine_flag(["injectra", "targets", "--machine"]));
        assert!(!has_machine_flag(["--machine", "targets"]));
        assert!(!has_machine_flag(["injectra", "eval", "--", "--machine"]));
    }

    #[test]
    fn machine_exit_codes_and_envelopes_preserve_certainty_and_causes() {
        assert_eq!(machine_exit_code(Certainty::NotExecuted), ExitCode::from(3));
        assert_eq!(machine_exit_code(Certainty::Executed), ExitCode::from(4));
        assert_eq!(machine_exit_code(Certainty::Unknown), ExitCode::from(5));

        let ok = machine_ok_envelope(json!({"bundles": []}));
        assert_eq!(ok["ok"], json!(true));
        assert_eq!(ok["data"], json!({"bundles": []}));

        let err = Error::new(
            ErrorKind::Timeout,
            Phase::Execute,
            Certainty::Unknown,
            "timed out waiting for reply",
        )
        .caused_by(std::io::Error::other("read timed out"));
        let envelope = format_machine_error(&err);
        assert_eq!(envelope["ok"], json!(false));
        assert_eq!(envelope["error"]["kind"], json!("Timeout"));
        assert_eq!(envelope["error"]["phase"], json!("Execute"));
        assert_eq!(envelope["error"]["certainty"], json!("Unknown"));
        assert_eq!(
            envelope["error"]["message"],
            json!("timed out waiting for reply")
        );
        assert_eq!(envelope["error"]["causes"], json!(["read timed out"]));
    }

    #[test]
    fn read_eval_source_rejects_oversized_files() {
        let dir = std::env::temp_dir().join(format!("injectra-eval-test-{}", uuid_suffix()));
        fs::create_dir_all(&dir).unwrap();
        let ok_file = dir.join("ok.js");
        fs::write(&ok_file, "document.title").unwrap();
        assert_eq!(
            read_eval_source(None, Some(ok_file)).unwrap(),
            "document.title"
        );

        let big_file = dir.join("big.js");
        fs::write(&big_file, vec![b'a'; (EVAL_LIMIT as usize) + 1]).unwrap();
        assert!(read_eval_source(None, Some(big_file)).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    fn uuid_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
