//! macOS application metadata only. No browser family or debugging parameters.
use injectra_core::{Error, ErrorKind, Result};
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkInfo {
    pub dir_name: String,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub executable: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeSignSummary {
    pub signed: bool,
    pub adhoc: bool,
    pub hardened_runtime: bool,
    pub sandboxed: bool,
    pub allow_dyld_env: bool,
    pub disable_library_validation: bool,
    pub get_task_allow: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationInfo {
    pub bundle_path: PathBuf,
    pub executable: PathBuf,
    pub bundle_id: Option<String>,
    pub version: Option<String>,
    pub frameworks: Vec<OsString>,
    pub framework_details: Vec<FrameworkInfo>,
    pub resources_dir: PathBuf,
    pub linked_libraries: Vec<String>,
    pub codesign: CodeSignSummary,
}

fn read_plist_raw(plist_path: &Path, key: &str) -> Option<String> {
    let escaped = key.replace('.', "\\.");
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", &escaped, "raw", "-o", "-"])
        .arg(plist_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn read_xml_bool(xml_bytes: &[u8], key: &str) -> bool {
    let escaped = key.replace('.', "\\.");
    let Ok(mut child) = Command::new("/usr/bin/plutil")
        .args(["-extract", &escaped, "raw", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(xml_bytes);
    }
    let Ok(output) = child.wait_with_output() else {
        return false;
    };
    output.status.success()
        && String::from_utf8(output.stdout)
            .map(|s| s.trim() == "true")
            .unwrap_or(false)
}

fn inspect_framework(app: &Path, dir: &Path, dir_name: String) -> FrameworkInfo {
    let base = {
        let current = dir.join("Versions/Current");
        if current.is_dir() {
            current
        } else {
            dir.to_path_buf()
        }
    };
    let plist = base.join("Resources/Info.plist");
    let (bundle_id, version, exe_name) = if plist.is_file() {
        (
            read_plist_raw(&plist, "CFBundleIdentifier"),
            read_plist_raw(&plist, "CFBundleShortVersionString")
                .or_else(|| read_plist_raw(&plist, "CFBundleVersion")),
            read_plist_raw(&plist, "CFBundleExecutable"),
        )
    } else {
        (None, None, None)
    };
    let executable = exe_name.and_then(|name| {
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return None;
        }
        let candidate = base.join(name).canonicalize().ok()?;
        if candidate.starts_with(app) && candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    });
    FrameworkInfo {
        dir_name,
        bundle_id,
        version,
        executable,
    }
}

fn inspect_linked_libraries(executable: &Path) -> Vec<String> {
    let Ok(output) = Command::new("/usr/bin/otool")
        .arg("-L")
        .arg(executable)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            Some(
                trimmed
                    .split(" (")
                    .next()
                    .unwrap_or(trimmed)
                    .trim()
                    .to_owned(),
            )
        })
        .collect()
}

fn inspect_codesign(app: &Path) -> CodeSignSummary {
    let mut summary = CodeSignSummary::default();
    if let Ok(output) = Command::new("/usr/bin/codesign")
        .args(["-dv"])
        .arg(app)
        .output()
    {
        summary.signed = output.status.success();
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            let trimmed = line.trim();
            if trimmed == "Signature=adhoc" {
                summary.adhoc = true;
            }
            if (trimmed.starts_with("CodeDirectory") || trimmed.contains("flags="))
                && trimmed.contains("(runtime)")
            {
                summary.hardened_runtime = true;
            }
        }
    }
    if let Ok(output) = Command::new("/usr/bin/codesign")
        .args(["-d", "--entitlements", "-", "--xml"])
        .arg(app)
        .output()
        && output.status.success()
        && !output.stdout.is_empty()
    {
        let xml = &output.stdout;
        summary.sandboxed = read_xml_bool(xml, "com.apple.security.app-sandbox");
        summary.allow_dyld_env = read_xml_bool(
            xml,
            "com.apple.security.cs.allow-dyld-environment-variables",
        );
        summary.disable_library_validation =
            read_xml_bool(xml, "com.apple.security.cs.disable-library-validation");
        summary.get_task_allow = read_xml_bool(xml, "com.apple.security.get-task-allow")
            || read_xml_bool(xml, "get-task-allow");
    }
    summary
}

pub fn inspect_app(app: &Path) -> Result<ApplicationInfo> {
    if !cfg!(target_os = "macos") {
        return Err(Error::prepare(
            ErrorKind::Unsupported,
            "macOS bundle metadata inspection supports macOS only",
        ));
    }
    let app = app.canonicalize()?;
    if app.extension().and_then(|s| s.to_str()) != Some("app") {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "expected a .app bundle",
        ));
    }
    let info_plist = app.join("Contents/Info.plist");
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", "CFBundleExecutable", "raw", "-o", "-"])
        .arg(&info_plist)
        .output()?;
    if !output.status.success() {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "cannot read CFBundleExecutable from Info.plist",
        ));
    }
    let name = String::from_utf8(output.stdout)
        .map_err(|e| Error::prepare(ErrorKind::InvalidInput, e.to_string()).caused_by(e))?;
    let name = name.trim();
    if name.is_empty() || name.contains('/') || name == "." || name == ".." {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "invalid bundle executable name",
        ));
    }
    let executable = app.join("Contents/MacOS").join(name).canonicalize()?;
    if !executable.starts_with(&app) || !executable.is_file() {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "bundle executable escapes bundle or is missing",
        ));
    }
    let bundle_id = read_plist_raw(&info_plist, "CFBundleIdentifier");
    let version = read_plist_raw(&info_plist, "CFBundleShortVersionString")
        .or_else(|| read_plist_raw(&info_plist, "CFBundleVersion"));

    let dir = app.join("Contents/Frameworks");
    let mut frameworks = Vec::new();
    let mut framework_details = Vec::new();
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let file_name = entry.file_name();
                let dir_name = file_name.to_string_lossy().into_owned();
                if dir_name.ends_with(".framework") {
                    framework_details.push(inspect_framework(&app, &path, dir_name));
                }
                frameworks.push(file_name);
            }
        }
    }
    let resources_dir = app.join("Contents/Resources");
    let linked_libraries = inspect_linked_libraries(&executable);
    let codesign = inspect_codesign(&app);

    Ok(ApplicationInfo {
        bundle_path: app,
        executable,
        bundle_id,
        version,
        frameworks,
        framework_details,
        resources_dir,
        linked_libraries,
        codesign,
    })
}
