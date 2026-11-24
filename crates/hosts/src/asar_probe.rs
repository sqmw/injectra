//! Pure-Rust read-only Electron ASAR and entry probing (zero Node dependency).
use serde::Serialize;
use serde_json::Value;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const MAX_HEADER_BYTES: usize = 16 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackageSummary {
    pub layout: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_entry: Option<String>,
    pub entry_kind: &'static str,
}

struct AsarArchive {
    data_offset: u64,
    header: Value,
}

fn is_safe_relative(rel: &str) -> bool {
    !rel.is_empty()
        && !rel.starts_with('/')
        && !rel.contains('\\')
        && !rel.contains('\0')
        && rel
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

impl AsarArchive {
    fn open(path: &Path) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut prefix = [0u8; 16];
        file.read_exact(&mut prefix).ok()?;
        let header_size = u32::from_le_bytes(prefix[4..8].try_into().ok()?);
        let json_len = u32::from_le_bytes(prefix[12..16].try_into().ok()?) as usize;
        if json_len == 0 || json_len > MAX_HEADER_BYTES || (header_size as usize) < json_len {
            return None;
        }
        let mut json_buf = vec![0u8; json_len];
        file.read_exact(&mut json_buf).ok()?;
        let header: Value = serde_json::from_slice(&json_buf).ok()?;
        let data_offset = 8u64.checked_add(u64::from(header_size))?;
        Some(Self {
            data_offset,
            header,
        })
    }

    fn read_entry(&self, asar_path: &Path, rel_path: &str) -> Option<Vec<u8>> {
        if !is_safe_relative(rel_path) {
            return None;
        }
        let mut node = &self.header;
        for part in rel_path.split('/') {
            node = node.get("files")?.get(part)?;
        }
        if node.get("files").is_some() || node.get("link").is_some() {
            return None;
        }
        let size = node.get("size")?.as_u64()?;
        if size > MAX_ENTRY_BYTES {
            return None;
        }
        if node.get("unpacked").and_then(Value::as_bool) == Some(true) {
            let unpacked_root = asar_path.with_extension("asar.unpacked");
            let candidate = unpacked_root.join(rel_path).canonicalize().ok()?;
            let root_real = unpacked_root.canonicalize().ok()?;
            if !candidate.starts_with(&root_real) {
                return None;
            }
            return read_bounded(&candidate, MAX_ENTRY_BYTES);
        }
        let offset: u64 = node.get("offset")?.as_str()?.parse().ok()?;
        let mut file = File::open(asar_path).ok()?;
        file.seek(SeekFrom::Start(self.data_offset.checked_add(offset)?))
            .ok()?;
        let mut buf = vec![0u8; size as usize];
        file.read_exact(&mut buf).ok()?;
        Some(buf)
    }
}

fn read_bounded(path: &Path, limit: u64) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    if !meta.is_file() || meta.len() > limit {
        return None;
    }
    let mut buf = Vec::with_capacity(meta.len() as usize);
    file.take(limit + 1).read_to_end(&mut buf).ok()?;
    if buf.len() as u64 > limit {
        return None;
    }
    Some(buf)
}

enum ReadEntry {
    Direct(Vec<u8>),
    NestedAsar(Vec<u8>),
}

fn read_dir_entry(base: &Path, rel_path: &str) -> Option<ReadEntry> {
    if !is_safe_relative(rel_path) {
        return None;
    }
    let parts: Vec<&str> = rel_path.split('/').collect();
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i].ends_with(".asar") {
            let mut archive_path = base.to_path_buf();
            for p in &parts[..=i] {
                archive_path.push(p);
            }
            if archive_path.is_file() {
                let real_base = base.canonicalize().ok()?;
                let real_archive = archive_path.canonicalize().ok()?;
                if !real_archive.starts_with(&real_base) {
                    return None;
                }
                let inner = parts[i + 1..].join("/");
                let archive = AsarArchive::open(&real_archive)?;
                return archive
                    .read_entry(&real_archive, &inner)
                    .map(ReadEntry::NestedAsar);
            }
        }
    }
    let real_base = base.canonicalize().ok()?;
    let candidate = base.join(rel_path).canonicalize().ok()?;
    if !candidate.starts_with(&real_base) {
        return None;
    }
    read_bounded(&candidate, MAX_ENTRY_BYTES).map(ReadEntry::Direct)
}

fn classify_entry(main: &str, is_module_pkg: bool, entry: &ReadEntry) -> (&'static str, bool) {
    let lower = main.to_ascii_lowercase();
    let is_nested = matches!(entry, ReadEntry::NestedAsar(_));
    let bytes = match entry {
        ReadEntry::Direct(b) | ReadEntry::NestedAsar(b) => b,
    };
    if lower.ends_with(".jsc") || lower.ends_with(".bin") {
        return ("bytecode-or-packed", is_nested);
    }
    if !(lower.ends_with(".js") || lower.ends_with(".cjs") || lower.ends_with(".mjs")) {
        return ("bytecode-or-packed", is_nested);
    }
    if lower.ends_with(".mjs") || (is_module_pkg && !lower.ends_with(".cjs")) {
        return ("esm", is_nested);
    }
    if std::str::from_utf8(bytes).is_err() || bytes.contains(&0) {
        return (
            if is_nested {
                "commonjs-delegate"
            } else {
                "bytecode-or-packed"
            },
            is_nested,
        );
    }
    if is_nested {
        ("commonjs-delegate", true)
    } else {
        ("commonjs", false)
    }
}

fn candidate_paths(main: &str) -> Vec<String> {
    let file_name = main.rsplit('/').next().unwrap_or(main);
    if file_name.contains('.') {
        vec![main.to_owned()]
    } else {
        vec![
            main.to_owned(),
            format!("{main}.js"),
            format!("{main}.cjs"),
            format!("{main}.mjs"),
            format!("{main}/index.js"),
        ]
    }
}

pub fn probe_electron_package(resources_dir: &Path) -> PackageSummary {
    let asar_path = resources_dir.join("app.asar");
    let dir_pkg_path = resources_dir.join("app/package.json");
    let has_asar = asar_path.is_file();
    let has_dir = dir_pkg_path.is_file();
    if has_asar && has_dir {
        return PackageSummary {
            layout: "ambiguous",
            main_entry: None,
            entry_kind: "unknown",
        };
    }
    if !has_asar && !has_dir {
        return PackageSummary {
            layout: "missing",
            main_entry: None,
            entry_kind: "missing",
        };
    }

    let (base_layout, pkg_bytes, archive_opt) = if has_asar {
        let Some(archive) = AsarArchive::open(&asar_path) else {
            return PackageSummary {
                layout: "asar",
                main_entry: None,
                entry_kind: "bytecode-or-packed",
            };
        };
        let Some(pkg) = archive.read_entry(&asar_path, "package.json") else {
            return PackageSummary {
                layout: "asar",
                main_entry: None,
                entry_kind: "missing",
            };
        };
        ("asar", pkg, Some(archive))
    } else {
        let Some(pkg) = read_bounded(&dir_pkg_path, MAX_ENTRY_BYTES) else {
            return PackageSummary {
                layout: "directory",
                main_entry: None,
                entry_kind: "missing",
            };
        };
        ("directory", pkg, None)
    };

    let Ok(pkg_json) = serde_json::from_slice::<Value>(&pkg_bytes) else {
        return PackageSummary {
            layout: base_layout,
            main_entry: None,
            entry_kind: "bytecode-or-packed",
        };
    };
    let is_module_pkg = pkg_json.get("type").and_then(Value::as_str) == Some("module");
    let raw_main = pkg_json
        .get("main")
        .and_then(Value::as_str)
        .unwrap_or("index.js")
        .trim_start_matches("./");
    if !is_safe_relative(raw_main) {
        return PackageSummary {
            layout: base_layout,
            main_entry: None,
            entry_kind: "missing",
        };
    }

    let app_dir = resources_dir.join("app");
    for candidate in candidate_paths(raw_main) {
        let found = if let Some(archive) = &archive_opt {
            archive
                .read_entry(&asar_path, &candidate)
                .map(ReadEntry::Direct)
        } else {
            read_dir_entry(&app_dir, &candidate)
        };
        if let Some(entry) = found {
            let (entry_kind, is_nested) = classify_entry(&candidate, is_module_pkg, &entry);
            return PackageSummary {
                layout: if is_nested {
                    "nested-asar"
                } else {
                    base_layout
                },
                main_entry: Some(candidate),
                entry_kind,
            };
        }
    }

    PackageSummary {
        layout: base_layout,
        main_entry: Some(raw_main.to_owned()),
        entry_kind: "missing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn build_asar(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut root_files = serde_json::Map::new();
        let mut payload = Vec::new();
        for (name, bytes) in files {
            let offset = payload.len().to_string();
            payload.extend_from_slice(bytes);
            root_files.insert(
                (*name).to_owned(),
                json!({"size": bytes.len(), "offset": offset}),
            );
        }
        let header_json = serde_json::to_vec(&json!({"files": root_files})).unwrap();
        let json_len = header_json.len() as u32;
        let pad = (4 - (header_json.len() % 4)) % 4;
        let header_size = (8 + header_json.len() + pad) as u32;
        let mut out = Vec::new();
        out.extend_from_slice(&4u32.to_le_bytes());
        out.extend_from_slice(&header_size.to_le_bytes());
        out.extend_from_slice(&(header_size - 4).to_le_bytes());
        out.extend_from_slice(&json_len.to_le_bytes());
        out.extend_from_slice(&header_json);
        out.resize(out.len() + pad, 0);
        out.extend_from_slice(&payload);
        out
    }

    fn temp_resources(label: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("injectra-asar-probe-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn probes_asar_cjs_esm_bytecode_and_nested_asar() {
        let dir_cjs = temp_resources("cjs");
        let pkg = br#"{"name":"app","main":"main.js"}"#;
        let main = b"const { app } = require('electron');";
        fs::write(
            dir_cjs.join("app.asar"),
            build_asar(&[("package.json", pkg), ("main.js", main)]),
        )
        .unwrap();
        assert_eq!(
            probe_electron_package(&dir_cjs),
            PackageSummary {
                layout: "asar",
                main_entry: Some("main.js".into()),
                entry_kind: "commonjs",
            }
        );
        let _ = fs::remove_dir_all(&dir_cjs);

        let dir_esm = temp_resources("esm");
        let pkg_esm = br#"{"name":"app","type":"module","main":"index.js"}"#;
        fs::write(
            dir_esm.join("app.asar"),
            build_asar(&[("package.json", pkg_esm), ("index.js", b"export {};")]),
        )
        .unwrap();
        assert_eq!(probe_electron_package(&dir_esm).entry_kind, "esm");
        let _ = fs::remove_dir_all(&dir_esm);

        let dir_jsc = temp_resources("jsc");
        let pkg_jsc = br#"{"name":"app","main":"main.jsc"}"#;
        fs::write(
            dir_jsc.join("app.asar"),
            build_asar(&[("package.json", pkg_jsc), ("main.jsc", &[0, 1, 2, 3])]),
        )
        .unwrap();
        assert_eq!(
            probe_electron_package(&dir_jsc).entry_kind,
            "bytecode-or-packed"
        );
        let _ = fs::remove_dir_all(&dir_jsc);

        let dir_nested = temp_resources("nested");
        fs::create_dir_all(dir_nested.join("app")).unwrap();
        fs::write(
            dir_nested.join("app/package.json"),
            br#"{"name":"nested","main":"core.asar/index.js"}"#,
        )
        .unwrap();
        fs::write(
            dir_nested.join("app/core.asar"),
            build_asar(&[("index.js", b"\xde\xad\xbe\xef")]),
        )
        .unwrap();
        assert_eq!(
            probe_electron_package(&dir_nested),
            PackageSummary {
                layout: "nested-asar",
                main_entry: Some("core.asar/index.js".into()),
                entry_kind: "commonjs-delegate",
            }
        );
        let _ = fs::remove_dir_all(&dir_nested);
    }
}
