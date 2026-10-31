//! Read-only launch planning. The user retains ownership of application processes.
use injectra_core::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub note: &'static str,
}

pub fn launch_plan(app: &Path, port: u16) -> Result<LaunchPlan> {
    if !cfg!(target_os = "macos") {
        return Err("launch-plan currently supports macOS only".into());
    }
    if port == 0 {
        return Err("debugging port must be nonzero".into());
    }
    let app = app.canonicalize()?;
    if app.extension().and_then(|s| s.to_str()) != Some("app") {
        return Err("expected a .app bundle".into());
    }
    let resources = app.join("Contents/Frameworks/Electron Framework.framework");
    if !resources.is_dir() {
        return Err(
            "standard Electron Framework not found; custom bundles are not yet supported".into(),
        );
    }
    let output = std::process::Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleExecutable", "raw", "-o", "-"])
        .arg(app.join("Contents/Info.plist"))
        .output()?;
    if !output.status.success() {
        return Err("cannot read CFBundleExecutable from Info.plist".into());
    }
    let name = String::from_utf8(output.stdout)?;
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name == "." || name == ".." {
        return Err("invalid bundle executable name".into());
    }
    let executable = app.join("Contents/MacOS").join(name).canonicalize()?;
    if !executable.starts_with(&app) || !executable.is_file() {
        return Err("bundle executable escapes bundle or is missing".into());
    }
    Ok(LaunchPlan {
        executable,
        arguments: vec![
            format!("--remote-debugging-port={port}"),
            "--remote-debugging-address=127.0.0.1".into(),
        ],
        note: "Plan only; does not launch, restart or modify the app. Verify loopback binding and target identity before injection.",
    })
}
