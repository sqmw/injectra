//! macOS application metadata only. No browser family or debugging parameters.
use injectra_core::{Error, ErrorKind, Result};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct ApplicationInfo {
    pub executable: PathBuf,
    pub frameworks: Vec<OsString>,
}

pub fn inspect_app(app: &Path) -> Result<ApplicationInfo> {
    if !cfg!(target_os = "macos") {
        return Err("launch-plan currently supports macOS only".into());
    }
    let app = app.canonicalize()?;
    if app.extension().and_then(|s| s.to_str()) != Some("app") {
        return Err("expected a .app bundle".into());
    }
    let output = std::process::Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleExecutable", "raw", "-o", "-"])
        .arg(app.join("Contents/Info.plist"))
        .output()?;
    if !output.status.success() {
        return Err("cannot read CFBundleExecutable from Info.plist".into());
    }
    let name = String::from_utf8(output.stdout)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name == "." || name == ".." {
        return Err("invalid bundle executable name".into());
    }
    let executable = app.join("Contents/MacOS").join(name).canonicalize()?;
    if !executable.starts_with(&app) || !executable.is_file() {
        return Err("bundle executable escapes bundle or is missing".into());
    }
    let dir = app.join("Contents/Frameworks");
    let mut frameworks = Vec::new();
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                frameworks.push(entry.file_name());
            }
        }
    }
    Ok(ApplicationInfo {
        executable,
        frameworks,
    })
}
