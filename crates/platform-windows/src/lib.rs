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
        || lower.contains("crashpad")
        || lower.contains("elevate")
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
    let mut primary_in_component: Vec<&PathBuf> = resources_dirs
        .iter()
        .filter_map(|res| res.parent())
        .flat_map(|comp| {
            files.iter().filter(move |p| {
                p.parent() == Some(comp) && is_executable_file(p) && !is_auxiliary_windows_exe(p)
            })
        })
        .collect();
    primary_in_component.sort();
    if let Some(found) = primary_in_component.first() {
        return Some((*found).clone());
    }

    let mut top_primary: Vec<&PathBuf> = files
        .iter()
        .filter(|p| {
            p.parent() == Some(root_dir) && is_executable_file(p) && !is_auxiliary_windows_exe(p)
        })
        .collect();
    top_primary.sort();
    if let Some(found) = top_primary.first() {
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
    resources_dirs.sort();

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
}
