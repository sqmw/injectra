//! Windows application layout and PE metadata only. No browser family or debugging parameters.
pub mod pe;

use injectra_core::{Error, ErrorKind, Result};
use pe::{PeArch, PeIntegrityRecord, inspect_pe};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct WindowsAppLayout {
    pub root_dir: PathBuf,
    pub component_dir: PathBuf,
    pub executable: PathBuf,
    pub pe_arch: Option<PeArch>,
    pub component_files: Vec<PathBuf>,
    pub resources_dirs: Vec<PathBuf>,
    pub asar_integrity: Vec<PeIntegrityRecord>,
}

/// Canonicalize a path while stripping Windows `\\?\` drive verbatim prefix so Rust paths
/// stay compatible with Node.js `fs.realpathSync`, Win32 process queries, and Chromium switches.
pub fn normalize_canonical_path(path: &Path) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)?;
    #[cfg(windows)]
    {
        if let Some(text) = canonical.to_str()
            && let Some(stripped) = text.strip_prefix(r"\\?\")
            && !stripped.starts_with(r"UNC\")
        {
            return Ok(PathBuf::from(stripped));
        }
    }
    Ok(canonical)
}

fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            return (meta.permissions().mode() & 0o111) != 0;
        }
    }
    false
}

fn is_auxiliary_windows_exe(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    lower.starts_with("unins")
        || lower.starts_with("uninstall")
        || lower == "update.exe"
        || lower == "squirrel.exe"
        || lower.ends_with("_executionstub.exe")
        || lower.ends_with("_proxy.exe")
        || lower.ends_with("_helper.exe")
        || lower.ends_with("_launcher.exe")
        || lower.ends_with("_guard.exe")
        || lower.ends_with("_downloader.exe")
        || lower.ends_with("_doctor.exe")
        || lower.ends_with("cmd.exe")
        || lower.contains("updater")
        || lower.contains("crashpad")
        || lower.contains("crash_handler")
        || lower.contains("crash_reporter")
        || lower.contains("dump_reporter")
        || lower.contains("elevat")
}

fn version_sort_key(path: &Path) -> Vec<u64> {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return Vec::new();
    };
    name.split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

fn file_size_or_zero(path: &Path) -> u64 {
    fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

fn stem_matches_dir_name(exe: &Path, dir: &Path) -> bool {
    match (
        exe.file_stem().and_then(|s| s.to_str()),
        dir.file_name().and_then(|s| s.to_str()),
    ) {
        (Some(stem), Some(dir_name)) => stem.eq_ignore_ascii_case(dir_name),
        _ => false,
    }
}

fn collect_shallow_files(root: &Path, max_depth: usize, max_entries: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if files.len() >= max_entries {
            break;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut children: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        children.sort();
        for path in children {
            if files.len() >= max_entries {
                break;
            }
            if path.is_symlink() {
                continue;
            }
            if path.is_dir() {
                if depth < max_depth {
                    stack.push((path, depth + 1));
                }
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn select_executable(
    root_dir: &Path,
    files: &[PathBuf],
    resources_dirs: &[PathBuf],
) -> Option<PathBuf> {
    let mut ranked: Vec<(bool, bool, u64, &PathBuf)> = Vec::new();
    for (idx, res) in resources_dirs.iter().enumerate() {
        let Some(comp) = res.parent() else {
            continue;
        };
        let is_newest_component = idx == 0;
        for file in files.iter().filter(|p| {
            p.parent() == Some(comp) && is_executable_file(p) && !is_auxiliary_windows_exe(p)
        }) {
            ranked.push((
                stem_matches_dir_name(file, root_dir),
                is_newest_component,
                file_size_or_zero(file),
                file,
            ));
        }
    }
    for file in files.iter().filter(|p| {
        p.parent() == Some(root_dir) && is_executable_file(p) && !is_auxiliary_windows_exe(p)
    }) {
        ranked.push((
            stem_matches_dir_name(file, root_dir),
            false,
            file_size_or_zero(file),
            file,
        ));
    }
    ranked.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.3.cmp(b.3))
    });
    if let Some((_, _, _, found)) = ranked.first() {
        return Some((*found).clone());
    }

    let mut top_any: Vec<&PathBuf> = files
        .iter()
        .filter(|p| p.parent() == Some(root_dir) && is_executable_file(p))
        .collect();
    top_any.sort();
    if let Some(found) = top_any.first() {
        return Some((*found).clone());
    }

    files.iter().find(|p| is_executable_file(p)).cloned()
}

pub fn inspect_layout(path: &Path) -> Result<WindowsAppLayout> {
    let canonical = normalize_canonical_path(path)?;
    let (root_dir, explicit_exe) = if canonical.is_file() {
        let parent = canonical.parent().ok_or_else(|| {
            Error::prepare(
                ErrorKind::InvalidInput,
                "executable has no parent directory",
            )
        })?;
        (parent.to_path_buf(), Some(canonical))
    } else if canonical.is_dir() {
        (canonical, None)
    } else {
        return Err(Error::prepare(
            ErrorKind::InvalidInput,
            "target path must be a directory or executable",
        ));
    };

    let component_files = collect_shallow_files(&root_dir, 3, 512);
    let mut resources_dirs = Vec::new();
    for file in &component_files {
        let name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name == "app.asar"
            && let Some(parent) = file.parent()
            && !resources_dirs.iter().any(|d| d == parent)
        {
            resources_dirs.push(parent.to_path_buf());
        } else if name == "package.json"
            && file
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                == Some("app")
            && let Some(res) = file.parent().and_then(|p| p.parent())
            && !resources_dirs.iter().any(|d| d == res)
        {
            resources_dirs.push(res.to_path_buf());
        }
    }
    resources_dirs.sort_by(|a, b| {
        let ka = a.parent().map(version_sort_key).unwrap_or_default();
        let kb = b.parent().map(version_sort_key).unwrap_or_default();
        kb.cmp(&ka).then_with(|| b.cmp(a))
    });

    let executable = explicit_exe
        .or_else(|| select_executable(&root_dir, &component_files, &resources_dirs))
        .ok_or_else(|| {
            Error::prepare(
                ErrorKind::Unsupported,
                "Unsupported: no executable or recognized Web UI host architecture in target path",
            )
        })?;

    let component_dir = executable
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root_dir.clone());

    let pe_summary = inspect_pe(&executable);
    let pe_arch = pe_summary.as_ref().map(|s| s.arch);
    let mut asar_integrity = pe_summary.map(|s| s.asar_integrity).unwrap_or_default();

    if asar_integrity.is_empty() {
        for file in component_files
            .iter()
            .filter(|p| {
                p.parent() == Some(component_dir.as_path())
                    && p.extension()
                        .and_then(|s| s.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case("dll"))
            })
            .take(32)
        {
            if let Some(summary) = inspect_pe(file)
                && !summary.asar_integrity.is_empty()
            {
                asar_integrity = summary.asar_integrity;
                break;
            }
        }
    }

    Ok(WindowsAppLayout {
        root_dir,
        component_dir,
        executable,
        pe_arch,
        component_files,
        resources_dirs,
        asar_integrity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_real_host_exe_over_squirrel_or_execution_stub_in_versioned_layout() {
        let dir = std::env::temp_dir().join(format!(
            "injectra-win-layout-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let ver_dir = dir.join("app-1.17.0");
        let res_dir = ver_dir.join("resources");
        fs::create_dir_all(&res_dir).unwrap();
        fs::write(dir.join("Update.exe"), b"MZ").unwrap();
        fs::write(ver_dir.join("squirrel.exe"), b"MZ").unwrap();
        fs::write(ver_dir.join("Host_ExecutionStub.exe"), b"MZ").unwrap();
        fs::write(ver_dir.join("Host.exe"), b"MZ").unwrap();
        fs::write(res_dir.join("app.asar"), b"asar").unwrap();

        let layout = inspect_layout(&dir).unwrap();
        assert_eq!(layout.executable.file_name().unwrap(), "Host.exe");
        assert_eq!(layout.resources_dirs.len(), 1);
        assert_eq!(layout.component_dir.file_name().unwrap(), "app-1.17.0");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn prefers_newest_version_and_root_named_host_over_helpers() {
        let dir = std::env::temp_dir().join(format!(
            "injectra-win-multiver-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let app_root = dir.join("SampleApp");
        let old_res = app_root.join("7.5.2.1065").join("resources");
        let new_res = app_root.join("7.5.4.1068").join("resources");
        fs::create_dir_all(&old_res).unwrap();
        fs::create_dir_all(&new_res).unwrap();
        fs::write(old_res.join("app.asar"), b"asar").unwrap();
        fs::write(new_res.join("app.asar"), b"asar").unwrap();
        fs::write(app_root.join("7.5.2.1065").join("apollocmd.exe"), b"MZ").unwrap();
        fs::write(app_root.join("7.5.4.1068").join("apollocmd.exe"), b"MZ").unwrap();
        fs::write(
            app_root.join("7.5.4.1068").join("sampleapp.exe"),
            vec![b'M', b'Z', 0, 0, 0, 0],
        )
        .unwrap();
        fs::write(app_root.join("sampleapp.exe"), b"MZ").unwrap();

        let layout = inspect_layout(&app_root).unwrap();
        assert_eq!(layout.executable.file_name().unwrap(), "sampleapp.exe");
        assert_eq!(layout.component_dir.file_name().unwrap(), "7.5.4.1068");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn falls_back_to_root_host_when_version_dir_only_has_helpers() {
        let dir = std::env::temp_dir().join(format!(
            "injectra-win-root-fallback-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let app_root = dir.join("StreamApp");
        let ver_res = app_root.join("8.8.0").join("resources");
        fs::create_dir_all(&ver_res).unwrap();
        fs::write(ver_res.join("app.asar"), b"asar").unwrap();
        fs::write(app_root.join("8.8.0").join("app_shell_updater.exe"), b"MZ").unwrap();
        fs::write(app_root.join("8.8.0").join("dump_reporter.exe"), b"MZ").unwrap();
        fs::write(app_root.join("streamapp.exe"), vec![b'M', b'Z', 1, 2]).unwrap();
        fs::write(app_root.join("streamapp_guard.exe"), vec![b'M', b'Z', 1]).unwrap();

        let layout = inspect_layout(&app_root).unwrap();
        assert_eq!(layout.executable.file_name().unwrap(), "streamapp.exe");

        let _ = fs::remove_dir_all(&dir);
    }
}
